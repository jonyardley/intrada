import SharedTypes
import SwiftUI

/// Every reason string is the core's: the shell renders, never composes
/// (design-principles T15).
struct UpNextHero: View {
  let plan: SuggestedPlan
  private let lead: SuggestedSession
  let onStart: () -> Void
  let onChange: () -> Void
  let onBuildOwn: () -> Void
  @Environment(\.marker) private var marker
  @Environment(\.heroGradient) private var heroGradient

  /// `nil` for a plan with no blocks, which the core never sends.
  init?(
    plan: SuggestedPlan, onStart: @escaping () -> Void, onChange: @escaping () -> Void,
    onBuildOwn: @escaping () -> Void
  ) {
    guard let lead = plan.blocks.first else { return nil }
    self.plan = plan
    self.lead = lead
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
    .background(heroGradient)
    .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.hero))
    .dropShadow(.hero)
    .accessibilityElement(children: .contain)
  }

  // Eyebrow, title, count and reason read as one sentence: split up, VoiceOver
  // announces "3 items · 15 min" detached from the piece it describes.
  private var headline: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
      ViewThatFits(in: .horizontal) {
        HStack(alignment: .firstTextBaseline) {
          eyebrowLabel.fixedSize()
          Spacer(minLength: IntradaSpacing.controlGap)
          countText.fixedSize()
        }
        VStack(alignment: .leading, spacing: 2) {
          eyebrowLabel
          countText.fixedSize(horizontal: false, vertical: true)
        }
      }

      Text(lead.pieceTitle)
        .font(IntradaFont.pageTitle(27))
        .foregroundStyle(IntradaColor.paperTop)
        .lineLimit(3)
        .minimumScaleFactor(0.75)
        .fixedSize(horizontal: false, vertical: true)

      HStack(alignment: .firstTextBaseline, spacing: 6) {
        if lead.priority {
          Image(systemName: "star.fill")
            .iconSize(.caption)
            .foregroundStyle(marker)
        }
        Text(lead.reason)
          .font(IntradaFont.subtitle)
          .foregroundStyle(IntradaColor.onAccent.opacity(IntradaOpacity.strong))
          .fixedSize(horizontal: false, vertical: true)
      }
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(headlineLabel)
  }

  private var eyebrowLabel: some View {
    Eyebrow(eyebrow, tint: IntradaColor.onAccent.opacity(IntradaOpacity.secondary))
  }

  private var countText: some View {
    Text(countLabel)
      .font(IntradaFont.meta)
      .foregroundStyle(IntradaColor.onAccent.opacity(IntradaOpacity.secondary))
  }

  private var itemList: some View {
    VStack(spacing: 0) {
      ForEach(Array(lead.items.enumerated()), id: \.element.itemId) { index, item in
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

  // Stacks when title and totals do not share a line, so neither is squeezed.
  private func laterRow(_ block: SuggestedSession) -> some View {
    let title = Text(block.pieceTitle)
      .font(IntradaFont.bodyMedium)
      .foregroundStyle(IntradaColor.paperTop)
    let totals = Text(blockLabel(block))
      .font(IntradaFont.meta)
      .foregroundStyle(IntradaColor.onAccent.opacity(IntradaOpacity.secondary))
    return ViewThatFits(in: .horizontal) {
      HStack(alignment: .firstTextBaseline, spacing: IntradaSpacing.controlGap) {
        title.fixedSize()
        Spacer(minLength: IntradaSpacing.controlGap)
        totals.fixedSize()
      }
      VStack(alignment: .leading, spacing: 2) {
        title.fixedSize(horizontal: false, vertical: true)
        totals.fixedSize(horizontal: false, vertical: true)
      }
      .frame(maxWidth: .infinity, alignment: .leading)
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
        titleLine(item)
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

  private func titleLine(_ item: SuggestedItem) -> Text {
    Text(verbatim: item.itemTitle)
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

  // The pair stacks whenever it does not fit one line: a stacked pair beats
  // either label truncating.
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

  private var spokenCount: String {
    "\(itemCountLabel), \(plan.estimatedMinutes) minutes"
  }

  private var headlineLabel: String {
    var parts = [eyebrow]
    // Filled to a length, the totals are the plan's, not the lead piece's.
    if plan.lengthMins != nil { parts.append(spokenCount) }
    parts.append(lead.pieceTitle)
    if let composer = lead.pieceSubtitle { parts.append(composer) }
    parts.append(spoken(lead.reason))
    if plan.lengthMins == nil { parts.append(spokenCount) }
    return parts.joined(separator: ", ")
  }

  private func rowLabel(_ item: SuggestedItem) -> String {
    var parts = [item.itemTitle]
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
