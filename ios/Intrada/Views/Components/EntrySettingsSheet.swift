import SharedTypes
import SwiftUI

/// Per-entry session-builder settings: an "Aim" note, the key and variation,
/// and a planned duration, opened by tapping a builder row. Every field pushes on
/// change (no separate Save), matching the `SessionSummaryScreen` notes idiom.
struct EntrySettingsSheet: View {
  let entry: SetlistEntryView
  let limits: LimitsView
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @Environment(\.dynamicTypeSize) private var typeSize

  @State private var intention: String
  @State private var hasPlannedDuration: Bool
  @State private var plannedMinutes: Int
  @State private var variantId: String?

  private var plannable: EntryVariationsView? {
    store.viewModel?.buildingSetlist?.entryVariations.first { $0.entryId == entry.id }
  }
  private var variants: [PickerVariationView] { plannable?.variations ?? [] }
  private var keys: EntryKeysView? {
    store.viewModel?.buildingSetlist?.entryKeys.first { $0.entryId == entry.id }
  }

  private var live: SetlistEntryView {
    store.viewModel?.buildingSetlist?.entries.first { $0.id == entry.id } ?? entry
  }

  var durationRange: ClosedRange<Int> {
    Int(limits.plannedDurationMinMins)...Int(limits.plannedDurationMaxMins)
  }

  static func initialPlannedMinutes(for entry: SetlistEntryView, limits: LimitsView) -> Int {
    entry.plannedDurationSecs.map { Int($0) / 60 } ?? Int(limits.plannedDurationDefaultMins)
  }

  init(entry: SetlistEntryView, limits: LimitsView) {
    self.entry = entry
    self.limits = limits
    _intention = State(initialValue: entry.intention ?? "")
    _hasPlannedDuration = State(initialValue: entry.plannedDurationSecs != nil)
    _plannedMinutes = State(initialValue: Self.initialPlannedMinutes(for: entry, limits: limits))
    _variantId = State(initialValue: entry.plannedVariationIds.first)
  }

  var body: some View {
    BottomSheet(title: entry.itemTitle, detents: [.medium, .large]) {
      ScrollView {
        VStack(alignment: .leading, spacing: IntradaSpacing.section) {
          aimSection
          if let keys {
            keySection(keys)
          }
          if !variants.isEmpty {
            stepSection
          }
          durationSection
          if live.removable {
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
          store.send(.session(.setEntryIntention(entryId: entry.id, intention: value)))
        }
    }
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

  private var durationSection: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
      settingToggle(
        "Planned duration", isOn: $hasPlannedDuration,
        identifier: "entrySettings.plannedDuration"
      )
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
