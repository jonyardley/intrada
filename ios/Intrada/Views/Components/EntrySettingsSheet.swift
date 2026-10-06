import SharedTypes
import SwiftUI

/// Per-entry session-builder settings: an "Aim" note, a rep-target counter, and
/// a planned duration — opened by tapping a builder row. Every field pushes on
/// change (no separate Save), matching the `SessionSummaryScreen` notes idiom.
struct EntrySettingsSheet: View {
  let entry: SetlistEntryView
  let limits: LimitsView
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @Environment(\.dynamicTypeSize) private var typeSize

  @State private var intention: String
  @State private var tracksReps: Bool
  @State private var repTarget: Int
  @State private var hasPlannedDuration: Bool
  @State private var plannedMinutes: Int
  @State private var variantId: String?

  private var plannable: EntryVariationsView? {
    store.viewModel?.buildingSetlist?.entryVariations.first { $0.entryId == entry.id }
  }
  private var variants: [PickerVariationView] { plannable?.variations ?? [] }
  private var sections: [SectionView] { plannable?.sections ?? [] }
  private var drills: [DrillOfferView] {
    store.viewModel?.buildingSetlist?.drillOffers.filter { $0.entryId == entry.id } ?? []
  }
  private var keys: EntryKeysView? {
    store.viewModel?.buildingSetlist?.entryKeys.first { $0.entryId == entry.id }
  }

  private var live: SetlistEntryView {
    store.viewModel?.buildingSetlist?.entries.first { $0.id == entry.id } ?? entry
  }

  var repTargetRange: ClosedRange<Int> { Int(limits.repTargetMin)...Int(limits.repTargetMax) }

  /// The core states the bound in seconds and the stepper offers whole minutes,
  /// so both ends round inwards: a bound the stepper cannot land on exactly
  /// would otherwise offer a minute the core refuses.
  var durationRange: ClosedRange<Int> {
    Int((limits.plannedDurationMinSecs + 59) / 60)...Int(limits.plannedDurationMaxSecs / 60)
  }

  static func initialRepTarget(for entry: SetlistEntryView, limits: LimitsView) -> Int {
    Int(entry.plannedRepTarget ?? limits.repTargetDefault)
  }

  static func initialPlannedMinutes(for entry: SetlistEntryView, limits: LimitsView) -> Int {
    Int((entry.plannedDurationSecs ?? limits.plannedDurationDefaultSecs) / 60)
  }

  init(entry: SetlistEntryView, limits: LimitsView) {
    self.entry = entry
    self.limits = limits
    _intention = State(initialValue: entry.intention ?? "")
    _tracksReps = State(initialValue: entry.plannedRepTarget != nil)
    _repTarget = State(initialValue: Self.initialRepTarget(for: entry, limits: limits))
    _hasPlannedDuration = State(initialValue: entry.plannedDurationSecs != nil)
    _plannedMinutes = State(initialValue: Self.initialPlannedMinutes(for: entry, limits: limits))
    _variantId = State(initialValue: entry.plannedVariationIds.first)
  }

  var body: some View {
    BottomSheet(title: entry.itemTitle, detents: [.medium, .large]) {
      ScrollView {
        VStack(alignment: .leading, spacing: IntradaSpacing.section) {
          aimSection
          focusSection
          if !sections.isEmpty {
            segmentsSection
          }
          if let keys {
            keySection(keys)
          }
          if !variants.isEmpty {
            stepSection
          }
          repsSection
          durationSection
          if entry.removable {
            removeButton
          }
        }
        .padding(IntradaSpacing.card)
      }
    }
  }

  private var aimSection: some View {
    FieldCard("Aim") {
      TextField("What are you aiming for on this one?", text: $intention, axis: .vertical)
        .lineLimit(2...4)
        .font(IntradaFont.body)
        .foregroundStyle(IntradaColor.ink)
        .onChange(of: intention) { _, value in
          let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
          let next = trimmed.isEmpty ? nil : trimmed
          guard next != entry.intention else { return }
          store.send(.session(.setEntryIntention(entryId: entry.id, intention: next)))
        }
      if let suggested = live.record.suggestedFocus {
        OfferChip("Focus: \(suggested.label)", systemImage: "plus") {
          setFocus(suggested.focus)
        }
        .accessibilityIdentifier("entrySettings.suggestedFocus")
      }
    }
  }

  private var focusSection: some View {
    EntryFocusCard(
      current: live.record.focus,
      choices: store.viewModel?.buildingSetlist?.focusChoices ?? [],
      sections: sections,
      send: setFocus)
  }

  private func setFocus(_ focus: IntentionFocus?) {
    store.send(.session(.setFocus(entryId: entry.id, focus: focus)))
  }

  private var segmentsSection: some View {
    EntrySegmentsCard(
      segments: live.record.segments,
      sections: sections,
      drills: drills,
      hasPlannedTime: live.plannedDurationSecs != nil,
      canAddSection: live.record.canAddSection,
      wholePiece: { store.send(.session(.setSegments(entryId: entry.id, segments: []))) },
      add: { store.send(.session(.addSegment(entryId: entry.id, sectionId: $0))) },
      remove: { store.send(.session(.removeSegment(entryId: entry.id, sectionId: $0))) },
      step: { store.send(.session(.stepSegment(entryId: entry.id, sectionId: $0, minutes: $1))) },
      tickDrill: tickDrill)
  }

  private func tickDrill(_ offer: DrillOfferView) {
    store.send(.session(Self.tickEvent(offer)))
  }

  static func tickEvent(_ offer: DrillOfferView) -> SessionEvent {
    offer.addedEntryId.map { .removeFromSetlist(entryId: $0) }
      ?? .addDrill(entryId: offer.entryId, exerciseId: offer.exerciseId)
  }

  private func keySection(_ keys: EntryKeysView) -> some View {
    let current = keys.currentLabel
    return NavigationLink {
      EntryKeyList(entryId: entry.id)
    } label: {
      HStack(spacing: IntradaSpacing.cardCompact) {
        FieldLabel("Key")
        Spacer(minLength: IntradaSpacing.controlGap)
        Text(current)
          .font(IntradaFont.body)
          .foregroundStyle(IntradaColor.ink)
          .multilineTextAlignment(.trailing)
        Image(systemName: "chevron.right")
          .iconSize(.caption, weight: .semibold)
          .foregroundStyle(IntradaColor.inkFaintIcon)
      }
      .frame(minHeight: 44)
      .contentShape(Rectangle())
    }
    .buttonStyle(.plain)
    .accessibilityLabel("Key")
    .accessibilityValue(current)
    .accessibilityIdentifier("entrySettings.key")
    .fieldCardSurface()
  }

  private var stepSection: some View {
    FieldCard("Variation") {
      Menu {
        Button("No variation") { plan(nil) }
        ForEach(variants, id: \.id) { variation in
          Button(variation.label) { plan(variation.id) }
        }
      } label: {
        HStack {
          Text(selectedVariationLabel)
            .font(IntradaFont.body)
            .foregroundStyle(IntradaColor.ink)
          Spacer()
          Image(systemName: "chevron.up.chevron.down")
            .imageScale(.small)
            .foregroundStyle(IntradaColor.inkFaintIcon)
        }
        .contentShape(Rectangle())
      }
      .accessibilityLabel("Variation: \(selectedVariationLabel)")
      .accessibilityHint("Choose a different variation")
      .buttonStyle(.plain)
    }
  }

  private var selectedVariationLabel: String {
    variants.first(where: { $0.id == variantId })?.label ?? "No variation"
  }

  private func plan(_ id: String?) {
    guard id != variantId else { return }
    variantId = id
    store.send(
      .session(.setEntryVariations(entryId: entry.id, variationIds: id.map { [$0] } ?? [])))
  }

  private var repsSection: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
      settingToggle("Track repetitions", isOn: $tracksReps, identifier: "entrySettings.trackReps")
        .onChange(of: tracksReps) { _, on in
          store.send(
            .session(.setRepTarget(entryId: entry.id, target: on ? UInt8(repTarget) : nil)))
        }
      if tracksReps {
        Stepper(value: $repTarget, in: repTargetRange) {
          Text("Target: \(repTarget) repetitions")
            .font(IntradaFont.body).foregroundStyle(IntradaColor.ink)
        }
        .onChange(of: repTarget) { _, value in
          guard tracksReps else { return }
          store.send(.session(.setRepTarget(entryId: entry.id, target: UInt8(value))))
        }
      }
    }
    .fieldCardSurface()
  }

  private var durationSection: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
      settingToggle("Planned duration", isOn: $hasPlannedDuration)
        .onChange(of: hasPlannedDuration) { _, on in
          store.send(
            .session(
              .setEntryDuration(
                entryId: entry.id, durationSecs: on ? UInt32(plannedMinutes * 60) : nil)))
        }
      if hasPlannedDuration {
        Stepper(value: $plannedMinutes, in: durationRange) {
          Text("\(plannedMinutes) min").font(IntradaFont.body).foregroundStyle(IntradaColor.ink)
        }
        .onChange(of: plannedMinutes) { _, value in
          guard hasPlannedDuration else { return }
          store.send(
            .session(.setEntryDuration(entryId: entry.id, durationSecs: UInt32(value * 60))))
        }
      }
    }
    .fieldCardSurface()
  }

  @ViewBuilder private func settingToggle(
    _ title: String, isOn: Binding<Bool>, identifier: String? = nil
  ) -> some View {
    if typeSize.isAccessibilitySize {
      VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
        FieldLabel(title)
          .accessibilityHidden(true)
        Toggle(title, isOn: isOn)
          .labelsHidden()
          .tint(IntradaColor.accent)
          .accessibilityIdentifier(identifier ?? "")
      }
    } else {
      Toggle(isOn: isOn) {
        FieldLabel(title)
      }
      .tint(IntradaColor.accent)
      .accessibilityIdentifier(identifier ?? "")
    }
  }

  // The row's non-gesture removal path (T4): dropping it from today's session
  // leaves the piece↔exercise relation untouched.
  private var removeButton: some View {
    Button(role: .destructive) {
      if store.send(.session(.removeFromSetlist(entryId: entry.id)), onSuccess: .impact) {
        dismiss()
      }
    } label: {
      Text("Remove from this session")
        .font(IntradaFont.bodyMedium)
        .frame(maxWidth: .infinity)
        .padding(.vertical, IntradaSpacing.cardCompact)
    }
    .buttonStyle(.plain)
    .foregroundStyle(IntradaColor.danger)
    .cardSurface(cornerRadius: IntradaRadius.control)
  }
}

#if DEBUG
  #Preview("Entry settings") {
    let store = Store.previewBuildingGrouped
    IntradaColor.sheetScrim.ignoresSafeArea()
      .sheet(isPresented: .constant(true)) {
        if let limits = store.viewModel?.limits {
          EntrySettingsSheet(entry: .previewGroupedScales, limits: limits).environment(store)
        }
      }
  }
#endif
