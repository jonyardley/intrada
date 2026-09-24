import SharedTypes
import SwiftUI

/// The bar under the tempo row: the metre and one dot per beat, filled where
/// the click sounds, with a ring travelling on the beat being heard. Filled
/// and hollow, never colour alone (T3). Tapping it opens the pattern sheet,
/// so the readout's start/stop tap is never contested (T19).
struct ClickBarLine: View {
  let metre: Metre
  let sounding: UInt16
  let currentBeat: Int?
  let onTap: () -> Void

  @ScaledMetric(relativeTo: .caption) private var dividerHeight: CGFloat = 16

  var body: some View {
    Button(action: onTap) {
      // A twelve-beat bar outgrows a phone's row, so the line tightens its
      // groups, then drops the metre, before it overflows (#2139).
      ViewThatFits(in: .horizontal) {
        row(.full)
        row(.compact)
        row(.bare)
      }
      .padding(.horizontal, IntradaSpacing.cardCompact)
      .frame(minHeight: 36)
      .background(IntradaColor.surfaceSunken, in: Capsule())
      .overlay(Capsule().strokeBorder(IntradaColor.divider, lineWidth: 1))
      .frame(minHeight: 44)
    }
    .buttonStyle(PressRebound())
    .accessibilityLabel("Metronome settings")
    .accessibilityValue(spokenValue)
    .accessibilityHint("Choose the time signature and which beats sound")
    .accessibilityIdentifier("click.bar")
  }

  private enum Fit { case full, compact, bare }

  private func row(_ fit: Fit) -> some View {
    HStack(spacing: IntradaSpacing.cardCompact) {
      if fit != .bare {
        Text(TempoUnit.metreLabel(metre))
          .font(IntradaFont.metaMedium)
          .monospacedDigit()
          .lineLimit(1)
          .fixedSize()
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      dots(fit)
      if fit == .full {
        HairlineDivider(axis: .vertical, colour: IntradaColor.divider)
          .frame(height: dividerHeight)
      }
      Image(systemName: "slider.horizontal.3")
        .iconSize(.inline)
        .foregroundStyle(IntradaColor.ink)
    }
  }

  private func dots(_ fit: Fit) -> some View {
    let beatGap = fit == .full ? IntradaSpacing.controlGap : IntradaSpacing.controlGap / 2
    return HStack(spacing: fit == .full ? IntradaSpacing.controlGap : IntradaSpacing.cardCompact) {
      ForEach(groupRanges.indices, id: \.self) { g in
        HStack(spacing: beatGap) {
          ForEach(groupRanges[g], id: \.self) { beat in
            BeatDot(sounding: sounds(beat), current: beat == currentBeat)
          }
        }
        if fit == .full && g < groupRanges.count - 1 {
          Spacer().frame(width: IntradaSpacing.cardCompact)
        }
      }
    }
  }

  /// One run per group (3+2+2 reads as the shape of the bar); a single run
  /// when the metre declares none.
  private var groupRanges: [Range<Int>] {
    let groups = metre.groups.map { $0.map(Int.init) } ?? [Int(metre.beats)]
    var ranges: [Range<Int>] = []
    var start = 0
    for group in groups {
      ranges.append(start..<(start + group))
      start += group
    }
    return ranges
  }

  private func sounds(_ beat: Int) -> Bool { sounding & (1 << UInt16(beat)) != 0 }

  var spokenValue: String {
    let beats = (0..<Int(metre.beats)).filter(sounds).map { "\($0 + 1)" }
    let sounded =
      beats.count == Int(metre.beats)
      ? "every beat" : "beat\(beats.count == 1 ? "" : "s") \(beats.joined(separator: ", "))"
    return "\(metre.beats) \(TempoUnit.spokenName(metre.unit)) beats, metronome on \(sounded)"
  }
}

private struct BeatDot: View {
  let sounding: Bool
  let current: Bool
  // Scales with Dynamic Type, capped so a twelve-beat bar still fits the row (#1950).
  @ScaledMetric(relativeTo: .caption) private var diameter: CGFloat = 10
  private var cappedDiameter: CGFloat { min(diameter, 14) }

  var body: some View {
    Circle()
      .fill(sounding ? IntradaColor.accent : Color.clear)
      .frame(width: cappedDiameter, height: cappedDiameter)
      .overlay(
        Circle().strokeBorder(
          sounding ? Color.clear : IntradaColor.slotOutline, lineWidth: 1.5)
      )
      .overlay(
        Circle()
          .strokeBorder(IntradaColor.ink, lineWidth: 1.5)
          .padding(-3)
          .opacity(current ? 1 : 0)
      )
  }
}

#if DEBUG
  #Preview("Bar lines") {
    VStack(spacing: 24) {
      ClickBarLine(
        metre: Metre(beats: 4, unit: 4, groups: nil), sounding: 0b1111, currentBeat: 1, onTap: {})
      ClickBarLine(
        metre: Metre(beats: 4, unit: 4, groups: nil), sounding: 0b1000, currentBeat: 3, onTap: {})
      ClickBarLine(
        metre: Metre(beats: 7, unit: 8, groups: [3, 2, 2]), sounding: 0b0101001, currentBeat: 0,
        onTap: {})
      ClickBarLine(
        metre: Metre(beats: 12, unit: 8, groups: [2, 2, 2, 2, 2, 2]), sounding: 0b0101_0101_0101,
        currentBeat: 0, onTap: {})
    }
    .padding()
    .background(RadialGradient.playerPaper)
  }
#endif
