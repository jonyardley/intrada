import SwiftUI

/// The in-the-moment repetition log — the "good friction" the player exists for.
/// Clean banks a rep (teal), Missed steps back (taupe, never shaming). Slot dots
/// fill toward the target so "how many to go" is always visible.
///
/// Dumb pipe: `count`/`target` come from the core's `ActiveSessionView`; the
/// buttons report intent up (`onClean`/`onMissed`) and the core re-derives the
/// count. No domain state lives here. Buttons disable at each clamp end.
struct RepCounter: View {
  let count: Int
  let target: Int
  let onClean: () -> Void
  let onMissed: () -> Void

  private var toGo: Int { max(0, target - count) }

  var body: some View {
    VStack(alignment: .leading, spacing: 12) {
      header
      dots
      buttons
    }
  }

  private var header: some View {
    HStack {
      Text("Repetitions")
        .font(IntradaFont.metaMedium)
        .foregroundStyle(IntradaColor.inkSecondary)
      Spacer()
      HStack(spacing: 0) {
        Text("\(count)")
          .fontWeight(.semibold)
          .foregroundStyle(IntradaColor.ink)
        Text(" of \(target) · \(toGo) to go")
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      .font(IntradaFont.meta)
      .monospacedDigit()
    }
  }

  private var dots: some View {
    HStack(spacing: 5) {
      ForEach(0..<max(target, 0), id: \.self) { i in
        RepDot(done: i < count)
          .popOnChange(count == i + 1)
      }
    }
  }

  private var buttons: some View {
    HStack(spacing: 10) {
      repButton(
        title: "Clean", icon: "checkmark", fg: IntradaColor.repCleanFg,
        bg: IntradaColor.repCleanBg, border: IntradaColor.repCleanBorder,
        disabled: count >= target, action: onClean)
      repButton(
        title: "Missed", icon: "xmark", fg: IntradaColor.repMissedFg,
        bg: IntradaColor.repMissedBg, border: IntradaColor.slotOutline,
        disabled: count <= 0, action: onMissed)
    }
  }

  private func repButton(
    title: String, icon: String, fg: Color, bg: Color, border: Color,
    disabled: Bool, action: @escaping () -> Void
  ) -> some View {
    Button(action: action) {
      HStack(spacing: 7) {
        Image(systemName: icon)
          .font(.system(size: 17, weight: .semibold))
        Text(title)
      }
      .font(IntradaFont.segment.weight(.semibold))
      .foregroundStyle(fg)
      .frame(maxWidth: .infinity)
      .padding(.vertical, 13)
      .background(bg)
      .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.control))
      .overlay(
        RoundedRectangle(cornerRadius: IntradaRadius.control).stroke(border, lineWidth: 1))
    }
    .buttonStyle(PressRebound())
    .disabled(disabled)
    .opacity(disabled ? 0.4 : 1)
    .accessibilityLabel(title)
    .accessibilityHint(
      title == "Clean" ? "Bank a clean repetition" : "Step back a repetition")
  }
}

private struct RepDot: View {
  let done: Bool

  var body: some View {
    Circle()
      .fill(done ? IntradaColor.successTeal : Color.clear)
      .frame(width: 11, height: 11)
      .overlay(
        Circle().strokeBorder(
          done ? Color.clear : IntradaColor.slotOutline, lineWidth: 1.6))
  }
}

#if DEBUG
  #Preview {
    struct Harness: View {
      @State private var count = 7
      let target = 12
      var body: some View {
        ZStack {
          PaperBackground()
          RepCounter(
            count: count, target: target,
            onClean: { if count < target { count += 1 } },
            onMissed: { if count > 0 { count -= 1 } }
          )
          .padding(IntradaSpacing.card)
        }
      }
    }
    return Harness()
  }
#endif
