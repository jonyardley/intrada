import SwiftUI

/// Tag editor for the add/edit forms: shows the current tags as removable chips
/// and a text field that suggests existing tags as you type (the same inline
/// reveal as `AutocompleteField`). Tapping a suggestion or pressing return adds
/// a chip; case-insensitive duplicates are ignored. The shell supplies the
/// suggestion pool (`ViewModel.availableTags`); this view owns no domain logic.
struct TagChipInput: View {
  let label: String
  @Binding var tags: [String]
  var suggestions: [String]

  @State private var draft = ""
  @FocusState private var focused: Bool

  /// Previews/snapshots can't drive `@FocusState`; forces the list open.
  var initiallyShowingSuggestions: Bool = false

  private var matches: [String] {
    let query = draft.trimmingCharacters(in: .whitespacesAndNewlines)
    let unused = suggestions.filter { suggestion in
      !tags.contains { $0.localizedCaseInsensitiveCompare(suggestion) == .orderedSame }
    }
    let pool =
      query.isEmpty
      ? unused
      : unused.filter {
        $0.localizedCaseInsensitiveContains(query)
          && $0.localizedCaseInsensitiveCompare(query) != .orderedSame
      }
    return Array(pool.prefix(6))
  }

  private var showSuggestions: Bool {
    (focused || initiallyShowingSuggestions) && !matches.isEmpty
  }

  var body: some View {
    VStack(spacing: 0) {
      VStack(alignment: .leading, spacing: 8) {
        Text(label)
          .font(IntradaFont.metaMedium)
          .foregroundStyle(IntradaColor.inkFaint)
        if !tags.isEmpty {
          FlowLayout(spacing: 6) {
            ForEach(tags, id: \.self) { tag in
              RemovableChip(text: tag) { remove(tag) }
            }
          }
        }
        TextField("Add a tag", text: $draft)
          .font(IntradaFont.field)
          .foregroundStyle(IntradaColor.ink)
          .textInputAutocapitalization(.never)
          .autocorrectionDisabled()
          .focused($focused)
          .submitLabel(.done)
          .onSubmit { add(draft) }
      }
      .padding(.vertical, 10)
      .padding(.horizontal, 16)
      .frame(maxWidth: .infinity, alignment: .leading)
      .background(IntradaColor.cardFill)
      .zIndex(1)

      if showSuggestions {
        suggestionList
          .transition(.move(edge: .top).combined(with: .opacity))
      }
    }
    .clipped()
    .animation(.snappy(duration: 0.22), value: showSuggestions)
  }

  private func add(_ raw: String) {
    let tag = raw.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !tag.isEmpty,
      !tags.contains(where: { $0.localizedCaseInsensitiveCompare(tag) == .orderedSame })
    else {
      draft = ""
      return
    }
    tags.append(tag)
    draft = ""
  }

  private func remove(_ tag: String) {
    tags.removeAll { $0 == tag }
  }

  private var suggestionList: some View {
    VStack(spacing: 0) {
      Rectangle().fill(IntradaColor.hairline).frame(height: 1)
      ForEach(matches, id: \.self) { suggestion in
        Button {
          add(suggestion)
          UISelectionFeedbackGenerator().selectionChanged()
        } label: {
          HStack(spacing: 10) {
            Image(systemName: "plus")
              .font(IntradaFont.meta)
              .foregroundStyle(IntradaColor.inkFaint)
            Text(suggestion)
              .font(IntradaFont.body)
              .foregroundStyle(IntradaColor.ink)
            Spacer(minLength: 0)
          }
          .padding(.vertical, 10)
          .padding(.horizontal, 16)
          .frame(maxWidth: .infinity, alignment: .leading)
          .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityHint("Adds the tag \(suggestion)")

        if suggestion != matches.last {
          Rectangle().fill(IntradaColor.hairline).frame(height: 1).padding(.leading, 16)
        }
      }
    }
    .background(IntradaColor.surfaceSunken.opacity(0.6))
  }
}

private struct RemovableChip: View {
  let text: String
  let onRemove: () -> Void

  var body: some View {
    HStack(spacing: 5) {
      Text(text)
        .font(IntradaFont.metaMedium)
        .foregroundStyle(IntradaColor.inkSecondary)
      Image(systemName: "xmark")
        .font(.system(size: 9, weight: .semibold))
        .foregroundStyle(IntradaColor.inkFaint)
    }
    .padding(.vertical, 4)
    .padding(.horizontal, 9)
    .background(IntradaColor.surfaceSunken, in: Capsule())
    .contentShape(Capsule())
    .onTapGesture(perform: onRemove)
    .accessibilityElement(children: .combine)
    .accessibilityLabel(text)
    // `.isButton` so VoiceOver advertises the tap as activatable (matches the
    // KeyPicker convention); a bare onTapGesture isn't surfaced reliably.
    .accessibilityAddTraits(.isButton)
    .accessibilityHint("Double tap to remove")
  }
}

#if DEBUG
  #Preview {
    struct Demo: View {
      @State private var tags = ["classical", "recital"]
      let pool = ["classical", "recital", "jazz", "warm-up", "technique", "etude"]
      var body: some View {
        ZStack {
          PaperBackground()
          ScrollView {
            VStack(spacing: 16) {
              VStack(spacing: 0) {
                TagChipInput(
                  label: "Tags", tags: $tags, suggestions: pool,
                  initiallyShowingSuggestions: true)
              }.cardSurface()
            }
            .padding(16)
          }
        }
      }
    }
    return Demo()
  }
#endif
