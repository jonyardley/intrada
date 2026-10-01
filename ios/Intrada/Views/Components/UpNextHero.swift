import SharedTypes
import SwiftUI

/// The Practice hero when the core has something to suggest (#1082): the lead
/// piece, why it is being suggested, its two or three items with their reasons,
/// any later blocks of today's plan (#57, #999) and one `Start · N min`. Every
/// reason string is the core's: the shell renders, never composes
/// (design-principles T15).
struct UpNextHero: View {
  let plan: SuggestedPlan
  private let suggestion: SuggestedSession
  let onStart: () -> Void
  let onChange: () -> Void
  let onBuildOwn: () -> Void
  @Environment(\.marker) private var marker

  /// `nil` for a plan with no blocks, which the core never sends.
  init?(
    plan: SuggestedPlan, onStart: @escaping () -> Void, onChange: @escaping () -> Void,
    onBuildOwn: @escaping () -> Void
  ) {
    guard let lead = plan.blocks.first else { return nil }
    self.plan = plan
    self.suggestion = lead
    self.onStart = onStart
    self.onChange = onChange
    self.onBuildOwn = onBuildOwn
  }

  private var laterBlocks: ArraySlice<SuggestedSession> { plan.blocks.dropFirst() }
  private var eyebrow: String { plan.lengthMins == nil ? "Up next" : "Today's plan" }

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.card) {
      headline
      itemList
      if !laterBlocks.isEmpty { laterList }
      startButton
      secondaryActions
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(IntradaSpacing.section)
    .background(LinearGradient.practiceHero)
    .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.hero))
    .dropShadow(.hero)
    .accessibilityElement(children: .contain)
  }

  // Eyebrow, title, count and reason read as one sentence: split up, VoiceOver
  // announces "3 items · 15 min" detached from the piece it describes.
  private var headline: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
      HStack(alignment: .firstTextBaseline) {
        Eyebrow(eyebrow, tint: IntradaColor.onAccent.opacity(IntradaOpacity.secondary))
        Spacer(minLength: IntradaSpacing.controlGap)
        Text(countLabel)
          .font(IntradaFont.meta)
          .foregroundStyle(IntradaColor.onAccent.opacity(IntradaOpacity.secondary))
      }

      Text(suggestion.pieceTitle)
        .font(IntradaFont.pageTitle(27))
        .foregroundStyle(IntradaColor.paperTop)
        .lineLimit(3)
        .minimumScaleFactor(0.75)
        .fixedSize(horizontal: false, vertical: true)

      HStack(alignment: .firstTextBaseline, spacing: 6) {
        if suggestion.priority {
          Image(systemName: "star.fill")
            .iconSize(.caption)
            .foregroundStyle(marker)
        }
        Text(suggestion.reason)
          .font(IntradaFont.subtitle)
          .foregroundStyle(IntradaColor.onAccent.opacity(IntradaOpacity.strong))
          .fixedSize(horizontal: false, vertical: true)
      }
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(headlineLabel)
  }

  private var itemList: some View {
    VStack(spacing: 0) {
      ForEach(Array(suggestion.items.enumerated()), id: \.element.itemId) { index, item in
        if index > 0 {
          Rectangle()
            .fill(IntradaColor.paperTop.opacity(IntradaOpacity.wash))
            .frame(height: 1)
        }
        row(item)
      }
    }
    .padding(.horizontal, IntradaSpacing.cardCompact)
    .background(IntradaColor.paperTop.opacity(IntradaOpacity.wash))
    .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
    .overlay(
      RoundedRectangle(cornerRadius: IntradaRadius.card)
        .strokeBorder(IntradaColor.paperTop.opacity(IntradaOpacity.wash), lineWidth: 1)
    )
  }

  private var laterList: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
      Eyebrow("Then", tint: IntradaColor.onAccent.opacity(IntradaOpacity.secondary))
      VStack(spacing: 0) {
        ForEach(Array(laterBlocks.enumerated()), id: \.element.pieceId) { index, block in
          if index > 0 {
            Rectangle()
              .fill(IntradaColor.paperTop.opacity(IntradaOpacity.wash))
              .frame(height: 1)
          }
          laterRow(block)
        }
      }
      .padding(.horizontal, IntradaSpacing.cardCompact)
      .overlay(
        RoundedRectangle(cornerRadius: IntradaRadius.card)
          .strokeBorder(IntradaColor.paperTop.opacity(IntradaOpacity.wash), lineWidth: 1)
      )
    }
  }

  private func laterRow(_ block: SuggestedSession) -> some View {
    HStack(alignment: .firstTextBaseline, spacing: IntradaSpacing.controlGap) {
      Text(block.pieceTitle)
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.paperTop)
        .fixedSize(horizontal: false, vertical: true)
      Spacer(minLength: IntradaSpacing.controlGap)
      Text(blockLabel(block))
        .font(IntradaFont.meta)
        .foregroundStyle(IntradaColor.onAccent.opacity(IntradaOpacity.secondary))
    }
    .padding(.vertical, IntradaSpacing.cardCompact)
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(
      "\(block.pieceTitle), \(blockLabel(block).replacingOccurrences(of: " · ", with: ", "))")
  }

  private func blockLabel(_ block: SuggestedSession) -> String {
    "\(itemsLabel(block.items.count)) · \(block.estimatedMinutes) min"
  }

  private func row(_ item: SuggestedItem) -> some View {
    HStack(alignment: .top, spacing: IntradaSpacing.controlGap) {
      Circle()
        .fill(item.itemType.onHeroAccent)
        .frame(width: 7, height: 7)
        .padding(.top, 6)

      VStack(alignment: .leading, spacing: 2) {
        Text(titleLine(item))
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.paperTop)
          .fixedSize(horizontal: false, vertical: true)
        Text(item.reason)
          .font(IntradaFont.meta)
          .foregroundStyle(IntradaColor.onAccent.opacity(IntradaOpacity.secondary))
          .fixedSize(horizontal: false, vertical: true)
      }
      Spacer(minLength: 0)
    }
    .padding(.vertical, IntradaSpacing.cardCompact)
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(rowLabel(item))
  }

  private func titleLine(_ item: SuggestedItem) -> AttributedString {
    var line = AttributedString(item.itemTitle)
    guard let variation = item.variantLabel else { return line }
    var suffix = AttributedString(" · \(variation)")
    suffix.foregroundColor = item.itemType.onHeroAccent
    line.append(suffix)
    return line
  }

  private var startButton: some View {
    Button(action: onStart) {
      HStack(spacing: IntradaSpacing.controlGap) {
        Image(systemName: "play.fill")
        Text("Start · \(plan.estimatedMinutes) min")
      }
      .font(IntradaFont.button)
      .foregroundStyle(IntradaColor.onMarker)
      .frame(maxWidth: .infinity)
      .padding(.vertical, IntradaSpacing.card)
      .background(
        marker, in: RoundedRectangle(cornerRadius: IntradaRadius.control))
    }
    .buttonStyle(PressRebound())
    .accessibilityLabel("Start practising")
    .accessibilityIdentifier("practice.start")
    .accessibilityValue("\(itemCountLabel), about \(plan.estimatedMinutes) minutes")
  }

  // ViewThatFits: at the largest text sizes the pair no longer fits one line,
  // and a stacked pair beats either label truncating.
  private var secondaryActions: some View {
    ViewThatFits(in: .horizontal) {
      HStack {
        changeButton
        Spacer(minLength: IntradaSpacing.controlGap)
        buildOwnButton
      }
      VStack(spacing: 0) {
        changeButton
        buildOwnButton
      }
    }
    .font(IntradaFont.subtitle)
    .foregroundStyle(IntradaColor.onAccent.opacity(IntradaOpacity.secondary))
  }

  private var changeButton: some View {
    Button("Change it first", action: onChange)
      .padding(.vertical, IntradaSpacing.controlGap)
      .accessibilityHint("Opens the session builder with this plan in it")
      .accessibilityIdentifier("practice.changePlan")
  }

  private var buildOwnButton: some View {
    Button("Build my own instead", action: onBuildOwn)
      .padding(.vertical, IntradaSpacing.controlGap)
      .accessibilityHint("Opens the session builder")
      .accessibilityIdentifier("practice.buildOwn")
  }

  private func itemsLabel(_ count: Int) -> String {
    "\(count) item\(count == 1 ? "" : "s")"
  }

  private var itemCountLabel: String { itemsLabel(Int(plan.itemCount)) }

  private var countLabel: String {
    "\(itemCountLabel) · \(plan.estimatedMinutes) min"
  }

  private var headlineLabel: String {
    var parts = [eyebrow, suggestion.pieceTitle]
    if let composer = suggestion.pieceSubtitle { parts.append(composer) }
    parts.append(spoken(suggestion.reason))
    parts.append(countLabel.replacingOccurrences(of: " · ", with: ", "))
    return parts.joined(separator: ", ")
  }

  private func rowLabel(_ item: SuggestedItem) -> String {
    var parts = [item.itemTitle]
    if let variation = item.variantLabel { parts.append("variation \(variation)") }
    parts.append(item.itemType.label)
    parts.append(spoken(item.reason))
    return parts.joined(separator: ", ")
  }

  // VoiceOver reads the house separator as "middle dot"; commas are the pause
  // the sentence actually wants.
  private func spoken(_ reason: String) -> String {
    reason.replacingOccurrences(of: " · ", with: ", ")
  }
}

#if DEBUG
  #Preview("Starred") {
    ZStack {
      PaperBackground()
      UpNextHero(plan: .previewStarred, onStart: {}, onChange: {}, onBuildOwn: {})?
        .padding(IntradaSpacing.card)
    }
  }

  #Preview("Filled to a length") {
    ZStack {
      PaperBackground()
      UpNextHero(plan: .previewFilled, onStart: {}, onChange: {}, onBuildOwn: {})?
        .padding(IntradaSpacing.card)
    }
  }

  #Preview("Unstarred, never marked") {
    ZStack {
      PaperBackground()
      UpNextHero(plan: .previewFresh, onStart: {}, onChange: {}, onBuildOwn: {})?
        .padding(IntradaSpacing.card)
    }
  }
#endif
