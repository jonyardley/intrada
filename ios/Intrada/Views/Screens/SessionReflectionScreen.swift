import SharedTypes
import SwiftUI

/// C2 — the reflection as a session closes, in the serif reserved for the
/// user's own words. No mic yet: it needs the capture effect #1309 tracks, so
/// the sentence arrives as text (keyboard dictation included).
struct SessionReflectionScreen: View {
  var onKeep: (String) -> Void
  var onDismiss: () -> Void

  @Environment(\.dynamicTypeSize) private var typeSize
  @State private var draft: String
  @FocusState private var writing: Bool

  init(onKeep: @escaping (String) -> Void, onDismiss: @escaping () -> Void) {
    self.onKeep = onKeep
    self.onDismiss = onDismiss
    _draft = State(initialValue: "")
  }

  #if DEBUG
    /// Snapshot seed: a reflection already dictated, without typing it.
    init(draft: String, onKeep: @escaping (String) -> Void, onDismiss: @escaping () -> Void) {
      self.onKeep = onKeep
      self.onDismiss = onDismiss
      _draft = State(initialValue: draft)
    }
  #endif

  private var kept: String { draft.trimmingCharacters(in: .whitespacesAndNewlines) }

  var body: some View {
    CoachCoverScaffold { scale in
      VStack(spacing: 0) {
        Text("SESSION DONE")
          .font(IntradaFont.ambientStrong(scale.eyebrow))
          .tracking(1.5)
          .foregroundStyle(IntradaColor.inkSecondary)
          .padding(.top, IntradaSpacing.cardCompact)
        Spacer(minLength: IntradaSpacing.section)
        ask(scale)
        Spacer(minLength: IntradaSpacing.section)
        footer
      }
    }
  }

  private func ask(_ scale: CoachScale) -> some View {
    VStack(spacing: IntradaSpacing.card) {
      Text("Anything worth keeping?")
        .font(IntradaFont.verdict(scale.ask))
        .foregroundStyle(IntradaColor.ink)
        .multilineTextAlignment(.center)
        .fixedSize(horizontal: false, vertical: true)
      // One gentle shape, never three fields. First to give way at accessibility
      // sizes, as the drill screen's identity detail is.
      if !typeSize.isAccessibilitySize {
        Text("What improved, what's still rough, what's next.")
          .font(IntradaFont.ambient(scale.support))
          .foregroundStyle(IntradaColor.inkSecondary)
          .multilineTextAlignment(.center)
          .fixedSize(horizontal: false, vertical: true)
      }
      field(scale)
    }
  }

  private func field(_ scale: CoachScale) -> some View {
    TextField("Say it however it comes…", text: $draft, axis: .vertical)
      .lineLimit(3...8)
      .font(IntradaFont.cardTitle(scale == .compact ? 17 : 22))
      .foregroundStyle(IntradaColor.ink)
      .focused($writing)
      .padding(IntradaSpacing.card)
      .background(
        IntradaColor.cardFill,
        in: RoundedRectangle(cornerRadius: IntradaRadius.panel, style: .continuous)
      )
      .overlay(
        RoundedRectangle(cornerRadius: IntradaRadius.panel, style: .continuous)
          .strokeBorder(IntradaColor.hairline, lineWidth: 1)
      )
      .accessibilityLabel("What's worth keeping from this session")
      .accessibilityHint("Dictate or type it. Kept with the session, and never scored")
      .toolbar {
        // A vertical-axis field takes the return key for a newline, so this is
        // the only way back to either answer with the keyboard up.
        ToolbarItemGroup(placement: .keyboard) {
          Spacer()
          Button("Done") { writing = false }
        }
      }
  }

  private var footer: some View {
    VStack(spacing: 0) {
      CoachAction(
        title: "Keep it", emphasis: .primary,
        hint: "Saves this note with the session",
        action: { onKeep(kept) }
      )
      .disabled(kept.isEmpty)
      .opacity(kept.isEmpty ? 0.45 : 1)
      // Respected without a follow-up nudge: reflections are always optional.
      CoachAction(title: "Not tonight", emphasis: .quiet, action: onDismiss)
        .padding(.top, IntradaSpacing.controlGap)
    }
  }
}

#if DEBUG
  #Preview("Nothing said yet") {
    SessionReflectionScreen(onKeep: { _ in }, onDismiss: {})
  }

  #Preview("Dictated") {
    SessionReflectionScreen(
      draft: "Stride's nearly there at 72. The bridge still rushes when I go from memory. "
        + "That's the thing to hit next.",
      onKeep: { _ in }, onDismiss: {})
  }

  #Preview("Largest accessibility size") {
    SessionReflectionScreen(onKeep: { _ in }, onDismiss: {})
      .environment(\.dynamicTypeSize, .accessibility5)
  }
#endif
