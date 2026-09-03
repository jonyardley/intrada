import SwiftUI

/// The pass counter, resident on the Focus Player and ignorable (T19). Until
/// the first tap it draws `slots` empty positions and no dots, because nothing
/// has been recorded; the core writes the target with the first tap.
///
/// Dumb pipe: every number comes from the core's `ActiveSessionView`, the
/// buttons report intent up, and the core re-derives the count. A miss is
/// always tappable below the target: whether a miss at zero counts is the
/// core's rule, and it records one.
struct RepCounter: View {
  let count: Int
  let slots: Int
  let touched: Bool
  let reached: Bool
  let onGotIt: () -> Void
  let onNotQuite: () -> Void

  @Environment(\.dynamicTypeSize) private var typeSize

  private var toGo: Int { max(0, slots - count) }
  private var stacked: Bool { typeSize.isAccessibilitySize }

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      header
      if touched {
        dots
      }
      buttons
    }
  }

  private var header: some View {
    HStack {
      Text("Passes")
        .font(IntradaFont.metaMedium)
        .foregroundStyle(IntradaColor.inkSecondary)
      Spacer()
      HStack(spacing: 0) {
        Text("\(count)")
          .fontWeight(.semibold)
          .foregroundStyle(IntradaColor.ink)
        Text(countTail)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      .font(IntradaFont.meta)
      .monospacedDigit()
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel("Passes")
    .accessibilityValue(spokenCount)
  }

  /// The "to go" clause is dropped at the target and before the first tap:
  /// neither state has a distance left to state.
  private var countTail: String {
    touched && !reached ? " of \(slots) · \(toGo) to go" : " of \(slots)"
  }

  private var spokenCount: String {
    touched && !reached ? "\(count) of \(slots), \(toGo) to go" : "\(count) of \(slots)"
  }

  private var dots: some View {
    HStack(spacing: 5) {
      ForEach(0..<max(slots, 0), id: \.self) { i in
        RepDot(done: i < count)
          .popOnChange(count == i + 1)
      }
    }
    .accessibilityHidden(true)
  }

  @ViewBuilder private var buttons: some View {
    if stacked {
      VStack(spacing: IntradaSpacing.controlGap) {
        gotIt
        notQuite(title: "Not quite right")
      }
    } else {
      HStack(spacing: IntradaSpacing.controlGap + 2) {
        gotIt
        notQuite(title: "Not quite")
      }
    }
  }

  private var gotIt: some View {
    repButton(
      title: "Got it", icon: "checkmark", fg: IntradaColor.repCleanFg,
      bg: IntradaColor.repCleanBg, border: IntradaColor.repCleanBorder,
      disabled: reached, action: onGotIt
    )
    .accessibilityLabel("Got it")
    .accessibilityHint("Banks a pass")
  }

  private func notQuite(title: String) -> some View {
    repButton(
      title: title, icon: "xmark", fg: IntradaColor.repMissedFg,
      bg: IntradaColor.repMissedBg, border: IntradaColor.slotOutline,
      disabled: reached, action: onNotQuite
    )
    .accessibilityLabel("Not quite")
    .accessibilityHint("Marks a pass not quite there")
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
          .lineLimit(1)
          .minimumScaleFactor(0.8)
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
      @State private var count = 0
      @State private var touched = false
      let slots = 10
      var body: some View {
        ZStack {
          PaperBackground()
          RepCounter(
            count: count, slots: slots, touched: touched, reached: count >= slots,
            onGotIt: {
              touched = true
              count = min(slots, count + 1)
            },
            onNotQuite: {
              touched = true
              count = max(0, count - 1)
            }
          )
          .padding(IntradaSpacing.card)
        }
      }
    }
    return Harness()
  }
#endif
