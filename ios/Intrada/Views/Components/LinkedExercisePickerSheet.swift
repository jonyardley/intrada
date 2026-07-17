import SharedTypes
import SwiftUI

/// Add/remove manager for a piece's related exercises. Lists every exercise
/// with the already-related ones pre-selected; tapping toggles membership and
/// "Done" hands the final set back — the caller links the added and unlinks the
/// removed. (Reorder stays in the detail card's Edit mode.)
///
/// The filter bar (star / sort / tag / search) drives *shell-local* state over
/// the passed-in `available` list — the picker curates its own subset rather
/// than the core's shared Library `ListQuery`, so filtering here never disturbs
/// the Library screen. A selected tray of removable chips sits above the list.
struct LinkedExercisePickerSheet: View {
  let available: [LibraryItemView]
  let linkedIds: [String]
  let onApply: (Swift.Set<String>) -> Void

  @Environment(\.dismiss) private var dismiss
  @State private var selected: Swift.Set<String>

  // Shell-local filter state (not the core ListQuery).
  @State private var priorityOnly = false
  @State private var sort = LibrarySort(field: .title, direction: .ascending)
  @State private var selectedTags: [String] = []
  @State private var filteringTags = false
  @State private var searchText = ""
  @State private var searchRevealed = false
  @FocusState private var searchFocused: Bool

  init(
    available: [LibraryItemView], linkedIds: [String],
    onApply: @escaping (Swift.Set<String>) -> Void
  ) {
    self.available = available
    self.linkedIds = linkedIds
    self.onApply = onApply
    _selected = State(initialValue: Swift.Set(linkedIds))
  }

  var body: some View {
    BottomSheet(
      title: "Add exercises",
      onDone: { onApply(selected) },
      leadingAction: { Button("Cancel") { dismiss() } },
      content: {
        if available.isEmpty {
          PlaceholderContent(
            systemImage: "music.note.list",
            message: "No exercises yet. Create an exercise to relate it to this piece.")
        } else {
          VStack(spacing: 0) {
            filterBar
            selectedTray
            list
          }
        }
      })
  }

  // ── Filter bar ──

  private var availableTags: [String] {
    var seen = Swift.Set<String>()
    var tags: [String] = []
    for tag in available.flatMap(\.tags) where seen.insert(tag.lowercased()).inserted {
      tags.append(tag)
    }
    return tags.sorted { $0.localizedCaseInsensitiveCompare($1) == .orderedAscending }
  }

  private var filterBar: some View {
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
          .foregroundStyle(priorityOnly ? IntradaColor.accent : IntradaColor.inkFaint)
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
        .foregroundStyle(selectedTags.isEmpty ? IntradaColor.inkFaint : IntradaColor.accent)
        .padding(IntradaSpacing.controlGap)
      }
      .buttonStyle(.plain)
      .accessibilityLabel("Filter by tag")
      .accessibilityValue(selectedTags.isEmpty ? "Off" : "\(selectedTags.count) selected")
      Button(action: toggleSearch) {
        Image(systemName: "magnifyingglass")
          .font(IntradaFont.tab)
          .foregroundStyle(searchRevealed ? IntradaColor.accent : IntradaColor.inkFaint)
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

  // ── Selected tray ──

  private var selectedItems: [LibraryItemView] {
    available.filter { selected.contains($0.id) }
  }

  @ViewBuilder private var selectedTray: some View {
    let items = selectedItems
    Group {
      if items.isEmpty {
        Text("No related exercises yet — tap to add.")
          .font(IntradaFont.meta)
          .foregroundStyle(IntradaColor.inkFaint)
          .frame(maxWidth: .infinity, alignment: .leading)
      } else {
        VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
          Text("\(items.count) related")
            .font(IntradaFont.eyebrow)
            .textCase(.uppercase)
            .kerning(1.2)
            .foregroundStyle(IntradaColor.inkFaint)
          ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 6) {
              ForEach(items, id: \.id) { exercise in
                TagChip(
                  exercise.title, style: .outlined,
                  onRemove: { toggle(exercise.id, isOn: true) })
              }
            }
          }
        }
      }
    }
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.top, IntradaSpacing.cardCompact)
    .padding(.bottom, IntradaSpacing.controlGap)
  }

  // ── List ──

  private var filtered: [LibraryItemView] {
    var items = available
    if priorityOnly { items = items.filter(\.priority) }
    if !selectedTags.isEmpty {
      items = items.filter { exercise in
        selectedTags.contains { tag in
          exercise.tags.contains { $0.localizedCaseInsensitiveCompare(tag) == .orderedSame }
        }
      }
    }
    let query = searchText.trimmingCharacters(in: .whitespaces)
    if !query.isEmpty {
      items = items.filter {
        $0.title.localizedCaseInsensitiveContains(query)
          || (metaLine($0)?.localizedCaseInsensitiveContains(query) ?? false)
      }
    }
    return items.sorted(by: isOrderedBefore)
  }

  private func isOrderedBefore(_ a: LibraryItemView, _ b: LibraryItemView) -> Bool {
    let ascending = sort.direction == .ascending
    switch sort.field {
    case .title:
      let result = a.title.localizedCaseInsensitiveCompare(b.title)
      return ascending ? result == .orderedAscending : result == .orderedDescending
    case .dateAdded:
      return ascending ? a.createdAt < b.createdAt : a.createdAt > b.createdAt
    case .lastPracticed:
      let la = a.practice?.lastPracticedAt ?? ""
      let lb = b.practice?.lastPracticedAt ?? ""
      return ascending ? la < lb : la > lb
    }
  }

  private var list: some View {
    ScrollView {
      VStack(spacing: 0) {
        createNewRow
        HairlineDivider().padding(.leading, IntradaSpacing.card)
        let rows = filtered
        if rows.isEmpty {
          Text("No exercises match your filters.")
            .font(IntradaFont.meta)
            .foregroundStyle(IntradaColor.inkSecondary)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, IntradaSpacing.card)
            .padding(.vertical, IntradaSpacing.row)
        } else {
          Text("Your exercises")
            .font(IntradaFont.eyebrow)
            .textCase(.uppercase)
            .kerning(1.2)
            .foregroundStyle(IntradaColor.inkFaint)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, IntradaSpacing.card)
            .padding(.top, IntradaSpacing.cardCompact)
            .padding(.bottom, IntradaSpacing.controlGap)
          ForEach(rows, id: \.id) { exercise in
            let isOn = selected.contains(exercise.id)
            Button {
              toggle(exercise.id, isOn: isOn)
            } label: {
              exerciseRow(exercise, isOn: isOn)
            }
            .buttonStyle(.plain)
            .accessibilityLabel(rowAccessibilityLabel(exercise, isOn: isOn))
            .accessibilityAddTraits(isOn ? [.isButton, .isSelected] : .isButton)

            if exercise.id != rows.last?.id {
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

  // ── Rows ──

  private var createNewRow: some View {
    NavigationLink(destination: LibraryAddScreen(defaultKind: .exercise)) {
      HStack(spacing: IntradaSpacing.cardCompact) {
        Image(systemName: "plus")
          .font(.system(size: 16, weight: .semibold))
          .foregroundStyle(IntradaColor.accent)
          .frame(width: 28, height: 28)
          .background(Circle().fill(IntradaColor.pieceBadgeBg))
          .accessibilityHidden(true)
        Text("Create new exercise")
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.accent)
        Spacer(minLength: 0)
      }
      .padding(.vertical, IntradaSpacing.cardCompact)
      .padding(.horizontal, IntradaSpacing.card)
      .frame(maxWidth: .infinity, alignment: .leading)
      .contentShape(Rectangle())
    }
    .buttonStyle(.plain)
    .accessibilityLabel("Create a new exercise")
  }

  private func exerciseRow(_ exercise: LibraryItemView, isOn: Bool) -> some View {
    HStack(spacing: IntradaSpacing.cardCompact) {
      ItemKind.exercise.bar
        .frame(width: 4, height: 30)
        .clipShape(Capsule())
      VStack(alignment: .leading, spacing: 3) {
        Text(exercise.title)
          .font(IntradaFont.cardTitle())
          .foregroundStyle(IntradaColor.ink)
        if let meta = metaLine(exercise) {
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
        .fill(isOn ? AnyShapeStyle(IntradaColor.exerciseAccent) : AnyShapeStyle(Color.clear))
        .overlay(
          Circle()
            .strokeBorder(IntradaColor.exerciseAccent, lineWidth: 2)
            .opacity(isOn ? 0 : 1))
      Image(systemName: isOn ? "checkmark" : "plus")
        .font(.system(size: 14, weight: .semibold))
        .foregroundStyle(isOn ? IntradaColor.onExercise : IntradaColor.exerciseAccent)
    }
    .frame(width: 28, height: 28)
  }

  // ── Helpers ──

  private func metaLine(_ exercise: LibraryItemView) -> String? {
    let parts = [exercise.keyDisplay, exercise.tempoDisplay]
      .compactMap { $0 }.filter { !$0.isEmpty }
    return parts.isEmpty ? nil : parts.joined(separator: " · ")
  }

  private func toggle(_ id: String, isOn: Bool) {
    if isOn {
      selected.remove(id)
    } else {
      selected.insert(id)
    }
    UISelectionFeedbackGenerator().selectionChanged()
  }

  private func rowAccessibilityLabel(_ exercise: LibraryItemView, isOn: Bool) -> String {
    var parts = [exercise.title]
    if let meta = metaLine(exercise) { parts.append(meta) }
    parts.append(isOn ? "related, tap to remove" : "not related, tap to add")
    return parts.joined(separator: ", ")
  }
}

#if DEBUG
  #Preview("Add or remove — one related") {
    LinkedExercisePickerSheet(
      available: [
        .previewExercise,
        LibraryItemView(
          id: "exercise-2", itemType: .exercise, title: "Db Major Scale", subtitle: "",
          key: "Db", modality: .major, tempo: nil, tempoMarking: nil, tempoBpm: nil,
          notes: nil, tags: [], createdAt: "", updatedAt: "", practice: nil,
          latestAchievedTempo: nil, priority: false, linkedExercises: [], linkedFromPieces: [],
          exerciseContexts: [], scaffoldPreview: nil, chordChart: nil),
        LibraryItemView(
          id: "exercise-3", itemType: .exercise, title: "Arpeggios in Db", subtitle: "",
          key: "Db", modality: .major, tempo: nil, tempoMarking: nil, tempoBpm: nil,
          notes: nil, tags: [], createdAt: "", updatedAt: "", practice: nil,
          latestAchievedTempo: nil, priority: false, linkedExercises: [], linkedFromPieces: [],
          exerciseContexts: [], scaffoldPreview: nil, chordChart: nil),
      ],
      linkedIds: ["exercise-1"],
      onApply: { _ in })
  }

  #Preview("Empty") {
    LinkedExercisePickerSheet(available: [], linkedIds: [], onApply: { _ in })
  }
#endif
