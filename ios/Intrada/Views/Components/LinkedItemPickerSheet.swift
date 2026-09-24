import IntradaCoreFFI
import SharedTypes
import SwiftUI

/// Add/remove manager for one side of the piece↔exercise link. Lists every
/// item of `kind` with the already-linked ones pre-selected; tapping toggles
/// membership and "Done" hands the final set back — the caller links the added
/// and unlinks the removed. (Reorder stays in the detail card's Edit mode.)
///
/// One component, both directions (#1363): `kind` carries the copy and the
/// type colour.
///
/// The filter bar (star / sort / tag / search) holds *shell-local* state rather
/// than the core's shared Library `ListQuery`, so filtering here never disturbs
/// the Library screen. The core narrows `library` to `kind` and applies the
/// star, tags, search and sort through the picker call (#1440, #1653, #1999).
///
/// For exercises, the picker also offers creating one inline (#1616): a
/// drafted exercise and a toggled selection hand back together on Done.
struct LinkedItemPickerSheet: View {
  let kind: ItemKind
  let library: [LibraryItemView]
  let linkedIds: [String]
  let onApply: (Swift.Set<String>, [StagedExercise]) -> Void

  @Environment(\.dismiss) private var dismiss
  @State private var selected: Swift.Set<String>
  @State private var drafts: [StagedExercise]
  @State private var creatingDraft = false

  // Shell-local filter state (not the core ListQuery).
  @State private var priorityOnly = false
  @State private var sort = LibrarySort(field: .title, direction: .ascending)
  @State private var selectedTags: [String] = []
  @State private var filteringTags = false
  @State private var searchText = ""
  @State private var searchRevealed = false
  @FocusState private var searchFocused: Bool

  init(
    kind: ItemKind, library: [LibraryItemView], linkedIds: [String],
    existingDrafts: [StagedExercise] = [],
    onApply: @escaping (Swift.Set<String>, [StagedExercise]) -> Void
  ) {
    self.kind = kind
    self.library = library
    self.linkedIds = linkedIds
    self.onApply = onApply
    _selected = State(initialValue: Swift.Set(linkedIds))
    _drafts = State(initialValue: existingDrafts)
  }

  // Only an exercise is light enough to draft inline; a piece needs the full form.
  private var allowsCreate: Bool { kind == .exercise }

  var body: some View {
    let ofKind = library.sortedAndFiltered(
      by: sort, search: "", filter: PickerFilterArg(kind: kind))
    BottomSheet(
      title: copy.sheetTitle,
      onDone: { onApply(selected, drafts) },
      leadingAction: { Button("Cancel") { dismiss() } },
      content: {
        if ofKind.isEmpty && !allowsCreate {
          PlaceholderContent(
            systemImage: kind.iconName, message: copy.noneAtAll)
        } else {
          VStack(spacing: 0) {
            if !ofKind.isEmpty { filterBar(availableTags: Self.tags(in: ofKind)) }
            selectedCount
            list(hasAnyOfKind: !ofKind.isEmpty)
          }
        }
      }
    )
    .sheet(isPresented: $creatingDraft) {
      DraftExerciseSheet(onDone: { drafts.append($0) })
    }
  }

  // ── Filter bar ──

  private static func tags(in items: [LibraryItemView]) -> [String] {
    var seen = Swift.Set<String>()
    var tags: [String] = []
    for tag in items.flatMap(\.tags) where seen.insert(tag.lowercased()).inserted {
      tags.append(tag)
    }
    return tags.sorted { $0.localizedCaseInsensitiveCompare($1) == .orderedAscending }
  }

  private func filterBar(availableTags: [String]) -> some View {
    VStack(spacing: 0) {
      // zIndex keeps the header above the revealed search bar so the bar slides
      // out from *under* it rather than ghosting over it (Design System Rules →
      // animated reveals).
      header.zIndex(1)
      if searchRevealed {
        LibrarySearchBar(text: $searchText, focused: $searchFocused, onCancel: cancelSearch)
          .padding(.horizontal, IntradaSpacing.card)
          .padding(.bottom, IntradaSpacing.cardCompact)
          .background(IntradaColor.paperTop)
          .transition(.move(edge: .top).combined(with: .opacity))
      }
    }
    .sensoryFeedback(.selection, trigger: searchRevealed)
    .sheet(isPresented: $filteringTags) {
      TagFilterSheet(
        available: availableTags, selected: selectedTags, onChange: { selectedTags = $0 })
    }
  }

  private var header: some View {
    HStack(spacing: IntradaSpacing.controlGap) {
      Button {
        priorityOnly.toggle()
      } label: {
        Image(systemName: priorityOnly ? "star.fill" : "star")
          .font(IntradaFont.tab)
          .foregroundStyle(priorityOnly ? IntradaColor.accent : IntradaColor.inkFaintIcon)
          .padding(.vertical, 6)
          .padding(.horizontal, 10)
          .overlay(Capsule().stroke(IntradaColor.divider, lineWidth: 1))
      }
      .buttonStyle(.plain)
      .accessibilityLabel("Show priorities only")
      .accessibilityAddTraits(priorityOnly ? [.isSelected] : [])
      Spacer(minLength: IntradaSpacing.controlGap)
      LibrarySortMenu(current: sort, onChange: { sort = $0 })
      Button {
        filteringTags = true
      } label: {
        Image(
          systemName: selectedTags.isEmpty
            ? "line.3.horizontal.decrease.circle"
            : "line.3.horizontal.decrease.circle.fill"
        )
        .font(IntradaFont.tab)
        .foregroundStyle(selectedTags.isEmpty ? IntradaColor.inkFaintIcon : IntradaColor.accent)
        .padding(IntradaSpacing.controlGap)
      }
      .buttonStyle(.plain)
      .accessibilityLabel("Filter by tag")
      .accessibilityValue(selectedTags.isEmpty ? "Off" : "\(selectedTags.count) selected")
      Button(action: toggleSearch) {
        Image(systemName: "magnifyingglass")
          .font(IntradaFont.tab)
          .foregroundStyle(searchRevealed ? IntradaColor.accent : IntradaColor.inkFaintIcon)
          .padding(IntradaSpacing.controlGap)
      }
      .buttonStyle(.plain)
      .accessibilityLabel("Search")
    }
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.top, IntradaSpacing.controlGap)
    .padding(.bottom, IntradaSpacing.cardCompact)
    .background(IntradaColor.paperTop)
    .overlay(alignment: .bottom) { HairlineDivider() }
  }

  private func toggleSearch() {
    if searchRevealed {
      cancelSearch()
    } else {
      withAnimation(IntradaMotion.standard) { searchRevealed = true }
      searchFocused = true
    }
  }

  private func cancelSearch() {
    searchText = ""
    searchFocused = false
    withAnimation(IntradaMotion.standard) { searchRevealed = false }
  }

  @ViewBuilder private var selectedCount: some View {
    let count = selected.count + drafts.count
    Group {
      if count == 0 {
        Text(copy.noneSelected)
          .font(IntradaFont.meta)
          .foregroundStyle(IntradaColor.inkSecondary)
      } else {
        Eyebrow("\(count) \(copy.selectedSuffix)", tint: IntradaColor.inkSecondary)
      }
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.top, IntradaSpacing.cardCompact)
    .padding(.bottom, IntradaSpacing.controlGap)
  }

  // ── List ──

  private var filtered: [LibraryItemView] {
    library.sortedAndFiltered(
      by: sort, search: searchText,
      filter: PickerFilterArg(kind: kind, priorityOnly: priorityOnly, tags: selectedTags))
  }

  private func list(hasAnyOfKind: Bool) -> some View {
    ScrollView {
      VStack(spacing: 0) {
        if allowsCreate {
          createTrigger
          HairlineDivider()
        }
        if !drafts.isEmpty {
          draftRows
          HairlineDivider()
        }
        let rows = filtered
        if rows.isEmpty {
          Text(hasAnyOfKind ? copy.noMatches : copy.noneAtAll)
            .font(IntradaFont.meta)
            .foregroundStyle(IntradaColor.inkSecondary)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, IntradaSpacing.card)
            .padding(.vertical, IntradaSpacing.card)
        } else {
          Eyebrow(copy.listHeading)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, IntradaSpacing.card)
            .padding(.top, IntradaSpacing.cardCompact)
            .padding(.bottom, IntradaSpacing.controlGap)
          ForEach(rows, id: \.id) { item in
            let isOn = selected.contains(item.id)
            Button {
              toggle(item.id, isOn: isOn)
            } label: {
              itemRow(item, isOn: isOn)
            }
            .buttonStyle(.plain)
            .accessibilityLabel(rowAccessibilityLabel(item, isOn: isOn))
            .accessibilityAddTraits(isOn ? [.isButton, .isSelected] : .isButton)

            if item.id != rows.last?.id {
              HairlineDivider().padding(.leading, IntradaSpacing.card)
            }
          }
        }
      }
      .cardSurface()
      .padding(IntradaSpacing.card)
    }
    .scrollDismissesKeyboard(.interactively)
  }

  private var createTrigger: some View {
    AddRowButton(title: "Create an exercise", style: .plain) { creatingDraft = true }
      .padding(.horizontal, IntradaSpacing.card)
  }

  private var draftRows: some View {
    VStack(spacing: 0) {
      Eyebrow("New")
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.top, IntradaSpacing.cardCompact)
        .padding(.bottom, IntradaSpacing.controlGap)
      ForEach(Array(drafts.enumerated()), id: \.element.id) { index, draft in
        if index > 0 {
          HairlineDivider().padding(.leading, IntradaSpacing.card)
        }
        DraftItemRow(
          title: draft.title, meta: draft.meta,
          onRemove: { drafts.removeAll { $0.id == draft.id } })
      }
    }
  }

  // ── Rows ──

  private func itemRow(_ item: LibraryItemView, isOn: Bool) -> some View {
    HStack(spacing: IntradaSpacing.cardCompact) {
      kind.bar
        .frame(width: 4, height: 30)
        .clipShape(Capsule())
      VStack(alignment: .leading, spacing: 3) {
        Text(item.title)
          .font(IntradaFont.cardTitle())
          .foregroundStyle(IntradaColor.ink)
        if let meta = metaLine(item) {
          Text(meta)
            .font(IntradaFont.meta)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
      }
      .frame(maxWidth: .infinity, alignment: .leading)
      membershipControl(isOn: isOn)
    }
    .padding(.vertical, IntradaSpacing.cardCompact)
    .padding(.horizontal, IntradaSpacing.card)
    .frame(maxWidth: .infinity, alignment: .leading)
    .contentShape(Rectangle())
  }

  private func membershipControl(isOn: Bool) -> some View {
    ZStack {
      Circle()
        .fill(isOn ? AnyShapeStyle(kind.accent) : AnyShapeStyle(Color.clear))
        .overlay(
          Circle()
            .strokeBorder(kind.accent, lineWidth: 2)
            .opacity(isOn ? 0 : 1))
      Image(systemName: isOn ? "checkmark" : "plus")
        .font(.system(size: 14, weight: .semibold))
        .foregroundStyle(isOn ? kind.onAccent : kind.accent)
    }
    .frame(width: 28, height: 28)
  }

  // ── Helpers ──

  /// A piece is told apart by its composer, an exercise by what it drills.
  private func metaLine(_ item: LibraryItemView) -> String? {
    let parts =
      kind == .piece
      ? [item.subtitle] : [item.keyDisplay, item.tempoDisplay].compactMap { $0 }
    let kept = parts.filter { !$0.isEmpty }
    return kept.isEmpty ? nil : kept.joined(separator: " · ")
  }

  private func toggle(_ id: String, isOn: Bool) {
    if isOn {
      selected.remove(id)
    } else {
      selected.insert(id)
    }
    UISelectionFeedbackGenerator().selectionChanged()
  }

  private func rowAccessibilityLabel(_ item: LibraryItemView, isOn: Bool) -> String {
    var parts = [item.title]
    if let meta = metaLine(item) { parts.append(meta) }
    parts.append(isOn ? copy.spokenOn : copy.spokenOff)
    return parts.joined(separator: ", ")
  }

  private var copy: PickerCopy { PickerCopy(kind: kind) }
}

/// Every string the picker changes between the two directions, in one place.
private struct PickerCopy {
  let kind: ItemKind

  var sheetTitle: String { kind == .piece ? "Link a piece" : "Add exercises" }

  var noneAtAll: String {
    kind == .piece ? "No pieces in your library yet." : "No exercises in your library yet."
  }

  var noneSelected: String {
    kind == .piece
      ? "Not tied to a piece yet · tap to link." : "No related exercises yet · tap to add."
  }

  var selectedSuffix: String { kind == .piece ? "linked" : "related" }

  var noMatches: String {
    kind == .piece ? "No pieces match the filters." : "No exercises match the filters."
  }

  var listHeading: String { kind == .piece ? "Pieces" : "Exercises" }

  var spokenOn: String { kind == .piece ? "linked, tap to unlink" : "related, tap to remove" }

  var spokenOff: String { kind == .piece ? "not linked, tap to link" : "not related, tap to add" }
}

#if DEBUG
  #Preview("Add or remove — one related") {
    LinkedItemPickerSheet(
      kind: .exercise,
      library: [
        .previewExercise,
        LibraryItemFixture.view(
          id: "exercise-2", itemType: .exercise, title: "Db Major Scale", key: "Db",
          modality: .major),
        LibraryItemFixture.view(
          id: "exercise-3", itemType: .exercise, title: "Arpeggios in Db", key: "Db",
          modality: .major),
      ],
      linkedIds: ["exercise-1"],
      onApply: { _, _ in })
  }

  #Preview("Empty") {
    LinkedItemPickerSheet(kind: .exercise, library: [], linkedIds: [], onApply: { _, _ in })
  }

  #Preview("Link a piece — from the exercise side") {
    LinkedItemPickerSheet(
      kind: .piece, library: [.previewPiece], linkedIds: [], onApply: { _, _ in })
  }
#endif
