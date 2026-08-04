import SwiftUI

/// Passive position against the click: which beat of the bar, which bar of the
/// phrase.
struct BeatPosition: View {
  /// One-based beat within the bar.
  let beat: Int
  let beatsPerBar: Int
  let bar: Int
  let bars: Int

  @Environment(\.coachScale) private var scale

  var body: some View {
    HStack(spacing: 10) {
      HStack(spacing: scale.pipGap) {
        ForEach(1...max(beatsPerBar, 1), id: \.self) { index in
          Circle()
            .fill(index == beat ? IntradaColor.accent : Color.clear)
            .frame(width: scale.pip, height: scale.pip)
            .overlay(
              Circle().strokeBorder(
                index == beat ? Color.clear : IntradaColor.slotOutline, lineWidth: 2))
        }
      }
      Text("bar \(bar) of \(bars)")
        .font(IntradaFont.ambient(scale.pipCaption))
        .monospacedDigit()
        .foregroundStyle(IntradaColor.inkSecondary)
        .dynamicTypeSize(...(.accessibility3))
    }
    .animation(nil, value: beat)
    .accessibilityElement(children: .ignore)
    .accessibilityLabel("Bar \(bar) of \(bars)")
  }
}

#if DEBUG
  #Preview {
    ZStack {
      RadialGradient.playerPaper.ignoresSafeArea()
      VStack(spacing: IntradaSpacing.section) {
        BeatPosition(beat: 2, beatsPerBar: 4, bar: 3, bars: 8)
        BeatPosition(beat: 4, beatsPerBar: 4, bar: 8, bars: 8)
        BeatPosition(beat: 1, beatsPerBar: 3, bar: 1, bars: 4)
          .environment(\.coachScale, .regular)
      }
    }
  }
#endif
