import SwiftUI

/// Resident and ignorable (T19). The full set of slots is drawn from the start,
/// empty ones outlined, so the row says what is left rather than growing as it
/// fills (#1735). Not quite stays tappable at zero because the core records a
/// miss there, and at the target because a miss steps the count back (#1507).
/// Got it counts past the target; undo takes back a tap without recording a
/// miss (#2107).
struct RepCounter: View {
  let count: Int
  let slots: Int
  let touched: Bool
  let reached: Bool
  var extra = 0
  var canUndo = false
  let onGotIt: () -> Void
  let onNotQuite: () -> Void
  var onUndo: () -> Void = {}

  @Environment(\.dynamicTypeSize) private var typeSize

  private let undoTarget: CGFloat = 44

  private var toGo: Int { max(0, slots - count) }
  private var stacked: Bool { typeSize.isAccessibilitySize }

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      header
      dots
      buttons
    }
  }

  @ViewBuilder private var header: some View {
    if stacked {
      VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
        HStack {
          label
          Spacer()
          undo
        }
        countText
      }
    } else {
      HStack {
        label
        Spacer()
        countText
        undo
      }
    }
  }

  private var label: some View {
    FieldLabel("Repetitions")
      .accessibilityHidden(true)
  }

  private var countText: some View {
    HStack(spacing: 0) {
      Text("\(count)")
        .fontWeight(.semibold)
        .foregroundStyle(IntradaColor.ink)
      Text(countTail)
        .foregroundStyle(IntradaColor.inkSecondary)
    }
    .font(IntradaFont.secondary)
    .monospacedDigit()
    .accessibilityElement(children: .ignore)
    .accessibilityLabel("Repetitions")
    .accessibilityValue(spokenCount)
    .accessibilityIdentifier("player.reps")
  }

  private var undo: some View {
    undoLabel
      .hidden()
      .overlay {
        if canUndo {
          Button(action: onUndo) { undoLabel }
            .buttonStyle(.plain)
            .accessibilityHint("Takes back the last repetition")
            .accessibilityIdentifier("player.undo")
        }
      }
      .padding(.vertical, -undoTarget / 2)
  }

  private var undoLabel: some View {
    Text("Undo")
      .font(IntradaFont.segment)
      .foregroundStyle(IntradaColor.accent)
      .frame(minWidth: undoTarget, minHeight: undoTarget)
      .contentShape(Rectangle())
  }

  private var countTail: String {
    if extra > 0 { return " of \(slots) · \(extra) extra" }
    return touched && !reached ? " of \(slots) · \(toGo) to go" : " of \(slots)"
  }

  private var spokenCount: String {
    if extra > 0 { return "\(count) of \(slots), \(extra) extra" }
    return touched && !reached ? "\(count) of \(slots), \(toGo) to go" : "\(count) of \(slots)"
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
      HStack(spacing: IntradaSpacing.controlGap) {
        notQuite(title: "Not quite")
        gotIt
      }
    }
  }

  private var gotIt: some View {
    repButton(
      title: "Got it", icon: "checkmark", fg: IntradaColor.repCleanFg,
      bg: IntradaColor.repCleanBg, border: IntradaColor.repCleanBorder,
      action: onGotIt
    )
    .accessibilityLabel("Got it")
    .accessibilityHint("Counts one repetition")
    .accessibilityIdentifier("player.gotIt")
  }

  private func notQuite(title: String) -> some View {
    repButton(
      title: title, icon: "xmark", fg: IntradaColor.repMissedFg,
      bg: IntradaColor.repMissedBg, border: IntradaColor.slotOutline,
      action: onNotQuite
    )
    .accessibilityLabel(title)
    .accessibilityHint("Takes one repetition off")
  }

  private func repButton(
    title: String, icon: String, fg: Color, bg: Color, border: Color,
    action: @escaping () -> Void
  ) -> some View {
    Button(action: action) {
      HStack(spacing: 7) {
        Image(systemName: icon)
          .font(IntradaFont.segment.weight(.semibold))
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
  }
}

private struct RepDot: View {
  let done: Bool
  // Scales with Dynamic Type, capped so ten passes still fit the row (#1950).
  @ScaledMetric(relativeTo: .caption) private var diameter: CGFloat = 11
  private var cappedDiameter: CGFloat { min(diameter, 18) }

  var body: some View {
    Circle()
      .fill(done ? IntradaColor.success : Color.clear)
      .frame(width: cappedDiameter, height: cappedDiameter)
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
            extra: max(0, count - slots), canUndo: touched,
            onGotIt: {
              touched = true
              count += 1
            },
            onNotQuite: {
              touched = true
              count = max(0, count - 1)
            },
            onUndo: { count = max(0, count - 1) }
          )
          .padding(IntradaSpacing.card)
        }
      }
    }
    return Harness()
  }
#endif
