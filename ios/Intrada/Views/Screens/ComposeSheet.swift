import SharedTypes
import SwiftUI

/// The steer sheet (#1256, A2 / A2r): "What do you want to practise today?"
///
/// A sheet *over* the prescribed screen, never a destination — dismissing it
/// lands back on the intact hero, and declining costs nothing. The price of
/// what has been added is stated on the primary action in the core's own
/// words; on a repeat visit that price is zero, which is the whole point of
/// resolution being paid once per item, ever.
struct ComposeSheet: View {
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss

  @State private var typed = ""
  @State private var resolving = false

  private var compose: ComposeView? { store.viewModel?.built.compose }
  private var items: [LibraryItemView] { store.viewModel?.items ?? [] }

  /// Library rows whose title contains what has been typed. Suggestions only:
  /// tapping one reports *which row* to the core, which still decides what it
  /// resolves to.
  private var suggestions: [LibraryItemView] {
    let wanted = typed.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
    guard wanted.count >= 2 else { return [] }
    return items.filter { $0.title.lowercased().contains(wanted) }.prefix(4).map { $0 }
  }

  var body: some View {
    NavigationStack {
      ZStack {
        PaperBackground()
        content
      }
      .navigationTitle("Today")
      .navigationBarTitleDisplayMode(.inline)
      .toolbar {
        ToolbarItem(placement: .topBarLeading) {
          Button {
            cancel()
          } label: {
            Image(systemName: "xmark")
          }
          .accessibilityLabel("Back to today's plan")
        }
      }
    }
    .fullScreenCover(isPresented: $resolving) {
      ResolutionFlow(onFinished: { resolving = false })
    }
  }

  private var content: some View {
    VStack(spacing: 0) {
      ScrollView {
        VStack(alignment: .leading, spacing: IntradaSpacing.row) {
          heading
          field
          if !suggestions.isEmpty { suggestionList }
          if let compose, !compose.entries.isEmpty { list(compose) }
        }
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.top, IntradaSpacing.card)
        .padding(.bottom, IntradaSpacing.section)
      }
      .scrollEdgeShadow()
      primaryAction
    }
  }

  private var heading: some View {
    VStack(alignment: .leading, spacing: 6) {
      Text("What do you want to practise today?")
        .font(IntradaFont.pageTitle(24))
        .foregroundStyle(IntradaColor.ink)
        .fixedSize(horizontal: false, vertical: true)
      Text("From your lesson, from your list — it all still counts.")
        .font(IntradaFont.subtitle)
        .foregroundStyle(IntradaColor.inkSecondary)
        .fixedSize(horizontal: false, vertical: true)
    }
  }

  private var field: some View {
    HStack(spacing: IntradaSpacing.controlGap) {
      Image(systemName: "magnifyingglass")
        .foregroundStyle(IntradaColor.inkFaint)
      TextField("Add a piece, drill, or anything…", text: $typed)
        .font(IntradaFont.field)
        .foregroundStyle(IntradaColor.ink)
        .submitLabel(.done)
        .onSubmit { add(text: typed, pickedItemId: nil) }
    }
    .padding(.vertical, IntradaSpacing.cardCompact)
    .padding(.horizontal, IntradaSpacing.row - 2)
    .background(IntradaColor.cardFill, in: RoundedRectangle(cornerRadius: IntradaRadius.control))
    .overlay(
      RoundedRectangle(cornerRadius: IntradaRadius.control)
        .strokeBorder(IntradaColor.divider, lineWidth: 1))
  }

  private var suggestionList: some View {
    VStack(spacing: 0) {
      ForEach(Array(suggestions.enumerated()), id: \.element.id) { index, item in
        Button {
          add(text: item.title, pickedItemId: item.id)
        } label: {
          HStack(spacing: IntradaSpacing.cardCompact) {
            Image(systemName: item.itemType.iconName)
              .font(.system(size: 12))
              .foregroundStyle(item.itemType.accent)
              .frame(width: 16, alignment: .leading)
            Text(item.title)
              .font(IntradaFont.bodyMedium)
              .foregroundStyle(IntradaColor.ink)
            Spacer(minLength: IntradaSpacing.controlGap)
            Image(systemName: "plus")
              .font(.system(size: 12))
              .foregroundStyle(IntradaColor.accent)
          }
          .padding(.vertical, IntradaSpacing.cardCompact)
          .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        if index < suggestions.count - 1 { HairlineDivider() }
      }
    }
    .padding(.horizontal, IntradaSpacing.card)
    .cardSurface()
  }

  private func list(_ compose: ComposeView) -> some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      Eyebrow(listLabel(compose))
      VStack(spacing: IntradaSpacing.controlGap) {
        ForEach(compose.entries, id: \.id) { entry in
          row(entry)
        }
      }
    }
  }

  /// The label says out loud what the list costs, so "all known" is visible
  /// before the primary action confirms it.
  private func listLabel(_ compose: ComposeView) -> String {
    let count = compose.entries.count
    return compose.questions.isEmpty
      ? "Today's list · \(count) · all known" : "Today's list · \(count)"
  }

  private func row(_ entry: ComposeEntryView) -> some View {
    HStack(alignment: .top, spacing: IntradaSpacing.cardCompact) {
      VStack(alignment: .leading, spacing: 6) {
        HStack(spacing: IntradaSpacing.controlGap) {
          // An unresolved row wears the question rather than a kind chip: it
          // has no kind yet, and showing both would say the same thing twice.
          if entry.kind == .unresolved {
            Label("1 question", systemImage: "questionmark.circle")
              .font(IntradaFont.badge)
              .foregroundStyle(IntradaColor.journalBadgeFg)
              .padding(.vertical, 5)
              .padding(.horizontal, 10)
              .background(
                IntradaColor.journalBadgeBg,
                in: RoundedRectangle(cornerRadius: IntradaRadius.badge, style: .continuous))
            Spacer(minLength: IntradaSpacing.controlGap)
          } else {
            TypeBadge(kind: entry.kind)
            Spacer(minLength: IntradaSpacing.controlGap)
            Image(systemName: "checkmark")
              .font(.system(size: 12, weight: .semibold))
              .foregroundStyle(IntradaColor.successTeal)
              .accessibilityLabel("Ready")
          }
        }
        Text(entry.name)
          .font(IntradaFont.cardTitle(16))
          .foregroundStyle(IntradaColor.ink)
          .fixedSize(horizontal: false, vertical: true)
        if let note = entry.note {
          Text(note)
            .font(IntradaFont.meta)
            .foregroundStyle(IntradaColor.inkSecondary)
            .fixedSize(horizontal: false, vertical: true)
        }
      }
      .frame(maxWidth: .infinity, alignment: .leading)
    }
    .padding(IntradaSpacing.cardCompact + 2)
    .cardSurface()
    .accessibilityElement(children: .combine)
    .accessibilityAction(named: "Remove") {
      store.send(.builtSession(.removeComposeEntry(entryId: entry.id)))
    }
    .swipeActionsCompatibleRemove {
      store.send(.builtSession(.removeComposeEntry(entryId: entry.id)))
    }
  }

  @ViewBuilder private var primaryAction: some View {
    if let compose, !compose.entries.isEmpty {
      VStack(spacing: 0) {
        Button(compose.buildLabel) {
          guard compose.canBuild else {
            resolving = true
            return
          }
          // Only dismiss on a build the core confirmed: an optimistic dismiss
          // over a refusal is the silent-no-op bug (#846).
          if store.send(.builtSession(.buildSession(source: nil)), onSuccess: .impact) {
            dismiss()
          }
        }
        .buttonStyle(PrimaryAction())
        // Declining costs nothing and the hero is untouched — said here, in
        // words, rather than left to a toolbar glyph.
        Button("Back to today's plan") { cancel() }
          .buttonStyle(QuietAction())
      }
      .padding(.horizontal, IntradaSpacing.card)
      .padding(.top, IntradaSpacing.cardCompact)
      .padding(.bottom, IntradaSpacing.controlGap)
      .background(IntradaColor.paperTop)
    }
  }

  private func cancel() {
    store.send(.builtSession(.cancelCompose))
    dismiss()
  }

  private func add(text: String, pickedItemId: String?) {
    let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty else { return }
    store.send(.builtSession(.addComposeEntry(text: trimmed, pickedItemId: pickedItemId)))
    typed = ""
  }
}

/// A full-width primary button in the brand gradient — the one action a screen
/// offers (docs/design-principles.md, one primary action per screen).
struct PrimaryAction: ButtonStyle {
  func makeBody(configuration: Configuration) -> some View {
    configuration.label
      .font(IntradaFont.bodyMedium)
      .foregroundStyle(IntradaColor.onAccent)
      .frame(maxWidth: .infinity, minHeight: 56)
      .background(LinearGradient.brandBar)
      .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.panel))
      .scaleEffect(configuration.isPressed ? 0.97 : 1)
      .animation(IntradaMotion.snappy, value: configuration.isPressed)
  }
}

extension View {
  /// Swipe-to-remove on a card that is not in a `List`. Kept as a modifier so
  /// the row reads as one thing and VoiceOver gets the same action.
  func swipeActionsCompatibleRemove(_ remove: @escaping () -> Void) -> some View {
    contextMenu {
      Button("Remove", systemImage: "trash", role: .destructive, action: remove)
    }
  }
}

#if DEBUG
  #Preview("First use") {
    ComposeSheet().environment(Store.previewComposing)
  }

  #Preview("Repeat use") {
    ComposeSheet().environment(Store.previewComposingAllKnown)
  }
#endif
