import AVFoundation
import Speech
import SwiftUI

// ── Merging ──

enum SpeechMerge {
  /// The spoken words after what is already written, word for word.
  static func merge(_ existing: String, _ spoken: String) -> String {
    let spoken = spoken.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !spoken.isEmpty else { return existing }
    guard let last = existing.last else { return spoken }
    return last.isWhitespace ? existing + spoken : existing + " " + spoken
  }
}

// ── Problems the musician sees ──

enum SpeechInputProblem: Equatable {
  case microphoneDenied
  case speechDenied
  case unavailable
  case nothingHeard

  var title: String {
    switch self {
    case .microphoneDenied: "Microphone access is off"
    case .speechDenied: "Speech recognition is off"
    case .unavailable: "Speaking isn't available"
    case .nothingHeard: "Nothing came through"
    }
  }

  var message: String {
    switch self {
    case .microphoneDenied, .speechDenied: "Turn it on in Settings to speak. You can still type."
    case .unavailable: "This device can't turn speech into text right now. You can still type."
    case .nothingHeard: "Try again, or type it instead."
    }
  }

  var offersSettings: Bool { self == .microphoneDenied || self == .speechDenied }
}

enum SpeechAccess: Equatable {
  case allowed
  case microphoneDenied
  case speechDenied
  case unavailable
}

/// What `SpeechInputModel` needs from the microphone and recogniser. Behind a
/// protocol because a simulator cannot dictate, so denial and failure are
/// otherwise untestable (the same reason as `PageCameraDevice`, #1460).
@MainActor
protocol SpeechRecogniser: AnyObject {
  func authorise() async -> SpeechAccess
  func start(
    onText: @escaping @MainActor @Sendable (String) -> Void,
    onEnd: @escaping @MainActor @Sendable (_ failed: Bool) -> Void) throws
  func stop()
}

// ── State ──

@Observable
@MainActor
final class SpeechInputModel {
  enum Phase: Equatable {
    case idle
    case starting
    case listening
  }

  private(set) var phase: Phase = .idle
  var problem: SpeechInputProblem?

  @ObservationIgnored private let recogniser: any SpeechRecogniser
  @ObservationIgnored private var base = ""
  @ObservationIgnored private var heard = ""
  @ObservationIgnored private var write: ((String) -> Void)?
  @ObservationIgnored private var attempt = 0

  init(recogniser: any SpeechRecogniser = LiveSpeechRecogniser()) {
    self.recogniser = recogniser
  }

  var isListening: Bool { phase == .listening }

  func toggle(current: String, write: @escaping (String) -> Void) async {
    switch phase {
    case .idle: await start(current: current, write: write)
    case .starting: return
    case .listening: stop()
    }
  }

  func start(current: String, write: @escaping (String) -> Void) async {
    guard phase == .idle else { return }
    phase = .starting
    switch await recogniser.authorise() {
    case .allowed: break
    case .microphoneDenied: return fail(.microphoneDenied)
    case .speechDenied: return fail(.speechDenied)
    case .unavailable: return fail(.unavailable)
    }
    guard phase == .starting else { return }
    attempt += 1
    let this = attempt
    base = current
    heard = ""
    self.write = write
    do {
      try recogniser.start(
        onText: { [weak self] text in self?.receive(text, attempt: this) },
        onEnd: { [weak self] failed in self?.ended(failed: failed, attempt: this) })
      phase = .listening
    } catch {
      report(error, "speech.start")
      self.write = nil
      fail(.unavailable)
    }
  }

  func sceneChanged(to scene: ScenePhase) {
    if scene == .background { stop() }
  }

  func stop() {
    guard phase != .idle else { return }
    if phase == .listening { recogniser.stop() }
    finish()
  }

  private func receive(_ text: String, attempt: Int) {
    guard attempt == self.attempt, phase == .listening else { return }
    heard = text
    write?(SpeechMerge.merge(base, text))
  }

  private func ended(failed: Bool, attempt: Int) {
    guard attempt == self.attempt, phase == .listening else { return }
    let nothing = heard.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    recogniser.stop()
    finish()
    if failed && nothing { problem = .nothingHeard }
  }

  private func finish() {
    attempt += 1
    write = nil
    phase = .idle
  }

  private func fail(_ problem: SpeechInputProblem) {
    phase = .idle
    self.problem = problem
  }
}

// ── The device ──

enum SpeechRecogniserError: Error {
  case unavailable
  case noMicrophoneInput
}

@MainActor
final class LiveSpeechRecogniser: SpeechRecogniser {
  private var recogniser: SFSpeechRecognizer?
  private var engine: AVAudioEngine?
  private var request: SFSpeechAudioBufferRecognitionRequest?
  private var task: SFSpeechRecognitionTask?

  func authorise() async -> SpeechAccess {
    let recogniser = recogniser ?? SFSpeechRecognizer()
    self.recogniser = recogniser
    guard let recogniser, recogniser.isAvailable else { return .unavailable }
    switch await Self.speechAuthorisation() {
    case .authorized: break
    case .denied, .restricted: return .speechDenied
    default: return .unavailable
    }
    guard await AVAudioApplication.requestRecordPermission() else { return .microphoneDenied }
    return .allowed
  }

  func start(
    onText: @escaping @MainActor @Sendable (String) -> Void,
    onEnd: @escaping @MainActor @Sendable (Bool) -> Void
  ) throws {
    guard let recogniser, recogniser.isAvailable else { throw SpeechRecogniserError.unavailable }
    try Self.activateSession()
    let engine = AVAudioEngine()
    let input = engine.inputNode
    let format = input.outputFormat(forBus: 0)
    // installTap raises an Objective-C exception on a zero-rate format, which
    // is what a device with no usable input reports.
    guard format.sampleRate > 0, format.channelCount > 0 else {
      Self.releaseSession()
      throw SpeechRecogniserError.noMicrophoneInput
    }

    let request = SFSpeechAudioBufferRecognitionRequest()
    request.shouldReportPartialResults = true
    request.addsPunctuation = true
    if recogniser.supportsOnDeviceRecognition { request.requiresOnDeviceRecognition = true }

    input.installTap(onBus: 0, bufferSize: 1024, format: format, block: Self.tap(request))
    engine.prepare()
    do {
      try engine.start()
    } catch {
      input.removeTap(onBus: 0)
      Self.releaseSession()
      throw error
    }
    self.engine = engine
    self.request = request
    task = recogniser.recognitionTask(
      with: request, resultHandler: Self.resultHandler(onText: onText, onEnd: onEnd))
  }

  func stop() {
    if let engine {
      engine.stop()
      engine.inputNode.removeTap(onBus: 0)
      Self.releaseSession()
    }
    request?.endAudio()
    task?.cancel()
    engine = nil
    request = nil
    task = nil
  }

  // Swift 6 isolates a closure formed on the main actor, and these callbacks run on
  // audio and Speech queues where that traps, so each is built nonisolated.

  private nonisolated static func speechAuthorisation() async
    -> SFSpeechRecognizerAuthorizationStatus
  {
    await withCheckedContinuation { continuation in
      SFSpeechRecognizer.requestAuthorization { continuation.resume(returning: $0) }
    }
  }

  private nonisolated static func tap(_ request: SFSpeechAudioBufferRecognitionRequest)
    -> AVAudioNodeTapBlock
  {
    { buffer, _ in request.append(buffer) }
  }

  private nonisolated static func resultHandler(
    onText: @escaping @MainActor @Sendable (String) -> Void,
    onEnd: @escaping @MainActor @Sendable (Bool) -> Void
  ) -> (SFSpeechRecognitionResult?, (any Error)?) -> Void {
    { result, error in
      let text = result?.bestTranscription.formattedString
      let isFinal = result?.isFinal ?? false
      let failed = error != nil
      Task { @MainActor in
        if let text { onText(text) }
        if isFinal || failed { onEnd(failed) }
      }
    }
  }

  // Off the main thread for the same reason as `ClickEngine.start`; the
  // category is put back to the click's own so a later click is not recorded over.
  private static func activateSession() throws {
    try DispatchQueue.global(qos: .userInitiated).sync {
      let session = AVAudioSession.sharedInstance()
      try session.setCategory(
        .playAndRecord, mode: .measurement, options: [.mixWithOthers, .defaultToSpeaker])
      try session.setActive(true)
    }
  }

  private static func releaseSession() {
    do {
      try DispatchQueue.global(qos: .userInitiated).sync {
        try AVAudioSession.sharedInstance().setCategory(
          .playback, mode: .default, options: [.mixWithOthers])
      }
    } catch {
      report(error, "speech.releaseSession")
    }
  }
}

// ── The control ──

/// A microphone button that speaks into any bound text, after what is there.
struct SpeechInputButton: View {
  @Binding var text: String
  let accessibilityLabel: String

  @Environment(\.marker) private var marker
  @Environment(\.scenePhase) private var scenePhase
  @State private var model: SpeechInputModel

  init(
    text: Binding<String>, accessibilityLabel: String = "Speak a note",
    recogniser: (any SpeechRecogniser)? = nil
  ) {
    _text = text
    self.accessibilityLabel = accessibilityLabel
    _model = State(initialValue: SpeechInputModel(recogniser: recogniser ?? LiveSpeechRecogniser()))
  }

  var body: some View {
    Button {
      Haptic.impact.play()
      let binding = $text
      Task { await model.toggle(current: text) { binding.wrappedValue = $0 } }
    } label: {
      Image(systemName: model.isListening ? "stop.fill" : "mic")
        .iconSize(.inline, weight: .medium)
        .foregroundStyle(model.isListening ? IntradaColor.onMarker : IntradaColor.inkSecondary)
        .frame(width: 36, height: 36)
        .background(Circle().fill(model.isListening ? marker : Color.clear))
        .contentShape(Circle())
        .frame(minWidth: 44, minHeight: 44)
    }
    .buttonStyle(.plain)
    .disabled(model.phase == .starting)
    .accessibilityLabel(model.isListening ? "Stop listening" : accessibilityLabel)
    .alert(
      model.problem?.title ?? "",
      isPresented: Binding(
        get: { model.problem != nil }, set: { if !$0 { model.problem = nil } }),
      presenting: model.problem
    ) { problem in
      if problem.offersSettings {
        Button("Open Settings", action: openSettings)
      }
      Button("OK", role: .cancel) {}
    } message: { problem in
      Text(problem.message)
    }
    .onDisappear { model.stop() }
    .onChange(of: scenePhase) { _, phase in model.sceneChanged(to: phase) }
  }

  private func openSettings() {
    guard let url = URL(string: UIApplication.openSettingsURLString) else { return }
    UIApplication.shared.open(url)
  }
}
