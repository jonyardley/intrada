// Phase C of intrada#309 — keeps the practice timer running while the
// device is locked / the app is backgrounded.
//
// Design (per specs/background-audio-plugin.md):
// - AVAudioSession.playback + .mixWithOthers so practice can run alongside
//   a backing track / metronome app the user is already playing.
// - 1s silent .wav looped via AVAudioPlayer to keep the OS from suspending
//   the app — required even with UIBackgroundModes: [audio] because iOS
//   suspends apps that have an active audio session but produce no samples.
// - MPNowPlayingInfoCenter so the lock screen shows the current item +
//   elapsed time + intrada artwork.
// - AVAudioSession.interruptionNotification re-arms the session after
//   phone calls / Siri so the timer resumes correctly mid-session.

import AVFoundation
import MediaPlayer
import SwiftRs
import Tauri
import UIKit
import WebKit

struct BeginSessionArgs: Decodable {
  let title: String
  let started_at: String  // RFC3339 UTC, parsed for elapsed-time math
}

struct NowPlayingArgs: Decodable {
  let title: String
  let position_label: String
  let started_at: String
}

class BackgroundAudioPlugin: Plugin {
  // The silent loop. Held for the lifetime of the session and torn down
  // on end_session — playback is the side-effect that keeps iOS from
  // suspending the app.
  //
  // All mutation goes through the main queue (see thread-safety note at
  // the top of the impl block). `MPNowPlayingInfoCenter` updates also
  // require the main thread per Apple's docs, so funnelling everything
  // there keeps both invariants.
  private var silentPlayer: AVAudioPlayer?
  private var currentItemStartedAt: Date?
  private var currentTitle: String?
  private var currentPositionLabel: String?

  // Cached because ISO8601DateFormatter is expensive to instantiate and
  // we re-parse on every `set_now_playing`.
  private static let rfc3339WithFractional: ISO8601DateFormatter = {
    let f = ISO8601DateFormatter()
    f.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    return f
  }()
  private static let rfc3339Plain: ISO8601DateFormatter = {
    let f = ISO8601DateFormatter()
    f.formatOptions = [.withInternetDateTime]
    return f
  }()

  override init() {
    super.init()
    NotificationCenter.default.addObserver(
      self,
      selector: #selector(handleAudioSessionInterruption(_:)),
      name: AVAudioSession.interruptionNotification,
      object: AVAudioSession.sharedInstance())
  }

  deinit {
    NotificationCenter.default.removeObserver(self)
  }

  // ── Commands ────────────────────────────────────────────────────────
  //
  // Tauri dispatches `Invoke` calls on a serial per-plugin queue, but the
  // interruption notification arrives on whatever thread NotificationCenter
  // chooses. To avoid races on the stored properties (and because
  // MPNowPlayingInfoCenter must be touched on the main thread anyway),
  // every state mutation hops to the main queue before reading or writing.

  @objc public func begin_session(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(BeginSessionArgs.self)
    let started = parseRfc3339(args.started_at)

    DispatchQueue.main.async { [weak self] in
      guard let self = self else { return }

      do {
        try self.activateAudioSession()
      } catch {
        // Session activation failed — surface to JS so Sentry catches
        // it, but the wall-clock timer keeps ticking regardless. Only
        // the lock-screen continuation is broken.
        invoke.reject(
          "background-audio: failed to activate audio session: \(error.localizedDescription)")
        return
      }

      // Don't throw on play() returning false — simulator silent mode and
      // some edge cases on device return false even when the session is
      // valid. The timer is the source of truth; lock-screen continuation
      // is best-effort.
      self.startSilentLoop()

      self.currentTitle = args.title
      self.currentPositionLabel = nil
      self.currentItemStartedAt = started
      self.seedNowPlaying()

      invoke.resolve()
    }
  }

  @objc public func set_now_playing(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(NowPlayingArgs.self)
    let started = parseRfc3339(args.started_at)

    DispatchQueue.main.async { [weak self] in
      guard let self = self else { return }
      self.currentTitle = args.title
      self.currentPositionLabel = args.position_label
      self.currentItemStartedAt = started
      self.seedNowPlaying()
      invoke.resolve()
    }
  }

  @objc public func end_session(_ invoke: Invoke) {
    DispatchQueue.main.async { [weak self] in
      guard let self = self else { return }

      self.silentPlayer?.stop()
      self.silentPlayer = nil

      // notifyOthersOnDeactivation lets a backing-track app resume its
      // own audio if it was ducked — common when practising along with
      // a recording.
      do {
        try AVAudioSession.sharedInstance().setActive(
          false, options: [.notifyOthersOnDeactivation])
      } catch {
        // Non-fatal — the next session start will reactivate.
      }

      MPNowPlayingInfoCenter.default().nowPlayingInfo = nil
      self.currentTitle = nil
      self.currentPositionLabel = nil
      self.currentItemStartedAt = nil

      invoke.resolve()
    }
  }

  // ── Internals ──────────────────────────────────────────────────────

  private func activateAudioSession() throws {
    let session = AVAudioSession.sharedInstance()
    try session.setCategory(
      .playback,
      mode: .default,
      // .mixWithOthers — practice along with a backing track the user
      // is already playing in another app (the actual use case). With
      // mixWithOthers, another app starting playback does NOT trigger
      // our interruptionNotification — only system events (calls /
      // Siri / alarms) do, which is the behaviour we want.
      options: [.mixWithOthers])
    try session.setActive(true)
  }

  /// Best-effort: start the silent loop so iOS keeps the audio session
  /// active. Failures are logged but don't abort `begin_session` — the
  /// wall-clock timer keeps running, only lock-screen continuation is
  /// degraded.
  private func startSilentLoop() {
    guard let url = Bundle.module.url(forResource: "silence", withExtension: "wav") else {
      // If this fires it means the resource bundle didn't ship — see
      // `Package.swift` `resources: [.process("Resources")]`. Logged
      // explicitly so a future maintainer can grep for it; not
      // user-visible since the bundle should always be present in a
      // built plugin.
      Logger.error("background-audio: silence.wav missing from plugin bundle")
      return
    }

    do {
      let player = try AVAudioPlayer(contentsOf: url)
      player.numberOfLoops = -1  // loop indefinitely
      player.volume = 0.0  // belt + braces; the file is already silent
      player.prepareToPlay()
      // play() can return false on simulator silent mode etc. — accept
      // it; the timer doesn't depend on actual playback succeeding.
      _ = player.play()
      self.silentPlayer = player
    } catch {
      Logger.error("background-audio: AVAudioPlayer init failed: \(error.localizedDescription)")
    }
  }

  /// Caller must already be on the main queue.
  private func seedNowPlaying() {
    guard let title = currentTitle else { return }

    var info: [String: Any] = [
      MPMediaItemPropertyTitle: title,
      MPNowPlayingInfoPropertyPlaybackRate: 1.0,
    ]

    // A non-nil playback duration is required for the lock-screen
    // scrubber to advance under playbackRate=1.0 — without it the
    // displayed elapsed time freezes between explicit pushes. We don't
    // model a planned duration in the plugin (intentions / per-item
    // durations live in core), so set a large synthetic duration. The
    // user never sees the duration directly; only the elapsed counter
    // ticking matters.
    info[MPMediaItemPropertyPlaybackDuration] = 24 * 60 * 60.0  // 24h

    if let positionLabel = currentPositionLabel {
      info[MPMediaItemPropertyArtist] = positionLabel  // e.g. "Item 2 of 5"
    }

    if let started = currentItemStartedAt {
      let elapsed = Date().timeIntervalSince(started)
      info[MPNowPlayingInfoPropertyElapsedPlaybackTime] = max(0.0, elapsed)
    }

    if let artwork = appArtwork() {
      info[MPMediaItemPropertyArtwork] = artwork
    }

    MPNowPlayingInfoCenter.default().nowPlayingInfo = info
  }

  /// Try to surface the app's icon as the Now Playing artwork.
  /// `UIImage(named: "AppIcon")` is the conventional path for asset-
  /// catalogue icons; falls back to walking `CFBundleIcons` for the
  /// case where the app uses the legacy `Icon-*` files; finally nil
  /// (lock screen renders a generic glyph).
  private func appArtwork() -> MPMediaItemArtwork? {
    if let image = UIImage(named: "AppIcon") {
      return MPMediaItemArtwork(boundsSize: image.size) { _ in image }
    }
    if let icons = Bundle.main.infoDictionary?["CFBundleIcons"] as? [String: Any],
      let primary = icons["CFBundlePrimaryIcon"] as? [String: Any],
      let files = primary["CFBundleIconFiles"] as? [String],
      let last = files.last,
      let image = UIImage(named: last)
    {
      return MPMediaItemArtwork(boundsSize: image.size) { _ in image }
    }
    return nil
  }

  @objc private func handleAudioSessionInterruption(_ notification: Notification) {
    guard
      let info = notification.userInfo,
      let raw = info[AVAudioSessionInterruptionTypeKey] as? UInt,
      let type = AVAudioSession.InterruptionType(rawValue: raw)
    else { return }

    switch type {
    case .began:
      // Phone call / Siri / alarm started — iOS suspends our session.
      // Nothing to do here; the silent loop pauses automatically.
      break
    case .ended:
      let optsRaw = info[AVAudioSessionInterruptionOptionKey] as? UInt ?? 0
      let opts = AVAudioSession.InterruptionOptions(rawValue: optsRaw)
      guard opts.contains(.shouldResume) else { return }

      // All shared-state mutation happens on the main queue. Refresh
      // the lock-screen card *first* so the visible state catches up
      // even if reactivation lags by a tick.
      DispatchQueue.main.async { [weak self] in
        guard let self = self else { return }
        self.seedNowPlaying()
        do {
          try AVAudioSession.sharedInstance().setActive(true)
          self.silentPlayer?.play()
        } catch {
          // Session re-activation failed — let it ride. The next
          // begin_session / set_now_playing will retry.
          Logger.error(
            "background-audio: failed to reactivate session after interruption: \(error.localizedDescription)"
          )
        }
      }
    @unknown default:
      break
    }
  }

  private func parseRfc3339(_ s: String) -> Date? {
    if let d = Self.rfc3339WithFractional.date(from: s) { return d }
    return Self.rfc3339Plain.date(from: s)
  }
}

@_cdecl("init_plugin_background_audio")
func initPlugin() -> Plugin {
  return BackgroundAudioPlugin()
}
