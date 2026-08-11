import SharedTypes
import SwiftUI

/// C2 — the reflection as a session closes. Asked once, answered either way,
/// and never asked again: the core decides whether it is offered at all, so
/// unmonitored play still ends without a question of any kind.
///
/// The words are the user's, so the field is set in serif — the voice reserved
/// for thoughts. **Audio is not kept yet**: the mic in the design frame needs
/// the capture effect #1309 tracks, so until then the sentence arrives as text
/// (the keyboard's own dictation included) and the row carries no audio path.
struct SessionReflectionScreen: View {
  var onKeep: (String) -> Void
  var onDismiss: () -> Void

  @Environment(\.horizontalSizeClass) private var sizeClass
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

  private var scale: CoachScale { sizeClass == .regular ? .regular : .compact }
  private var gutter: CGFloat { scale == .compact ? IntradaSpacing.card : IntradaSpacing.stage }
  private var kept: String { draft.trimmingCharacters(in: .whitespacesAndNewlines) }

  var body: some View {
    ZStack {
      RadialGradient.playerPaper.ignoresSafeArea()
      VStack(spacing: 0) {
        Text("SESSION DONE")
          .font(IntradaFont.ambientStrong(scale.eyebrow))
          .tracking(1.5)
          .foregroundStyle(IntradaColor.inkSecondary)
          .padding(.top, IntradaSpacing.cardCompact)
        Spacer(minLength: IntradaSpacing.card)
        ask
        Spacer(minLength: IntradaSpacing.card)
        footer
      }
      .padding(.horizontal, gutter)
      .padding(.bottom, IntradaSpacing.section)
    }
    .environment(\.coachScale, scale)
    .dynamicTypeSize(.xSmall ... .accessibility5)
  }

  private var ask: some View {
    VStack(spacing: IntradaSpacing.card) {
      Text("Anything worth keeping?")
        .font(IntradaFont.verdict(scale.question * 0.78))
        .foregroundStyle(IntradaColor.ink)
        .multilineTextAlignment(.center)
        .fixedSize(horizontal: false, vertical: true)
      // One gentle shape, offered as a prompt — never as three fields.
      Text("What improved, what's still rough, what's next.")
        .font(IntradaFont.ambient(scale == .compact ? 14 : 18))
        .foregroundStyle(IntradaColor.inkSecondary)
        .multilineTextAlignment(.center)
        .fixedSize(horizontal: false, vertical: true)
      field
    }
  }

  private var field: some View {
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
      .accessibilityHint("Dictate or type it. Nothing here changes what you practise next")
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
