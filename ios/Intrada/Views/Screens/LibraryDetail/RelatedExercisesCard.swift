import SharedTypes
import SwiftUI

struct RelatedExercisesCard: View {
  let item: LibraryItemView
  @Binding var editing: Bool
  let onAdd: () -> Void
  let onShowSuggestions: () -> Void

  @Environment(Store.self) private var store

  var body: some View {
    VStack(spacing: 0) {
      header
      if item.linkedExercises.isEmpty {
        SectionEmptyState(
          "Scales, arpeggios, and anything else you practise alongside this piece."
        ) {
          BrandBarButton(action: onAdd) {
            Image(systemName: "plus")
            Text("Add exercise")
          }
          .accessibilityLabel("Add an exercise for this piece")
          .accessibilityIdentifier("libraryDetail.addExercise")
        }
      } else {
        if !editing {
          // The rings below are each exercise's score *on this piece*, not its
          // overall. Say so, as the exercise hero's "Overall" (#1087 B2).
          Text("Marks shown are for this piece")
            .font(IntradaFont.meta)
            .foregroundStyle(IntradaColor.inkSecondary)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, IntradaSpacing.card)
            .padding(.bottom, IntradaSpacing.cardCompact)
        }
        rows
        populatedFooterActions
      }
      if item.scaffoldPreview != nil {
        HairlineDivider()
        suggestionsFromChart
      }
    }
    .cardSurface()
    .onChange(of: item.linkedExercises.isEmpty) { _, isEmpty in
      if isEmpty { editing = false }
    }
  }

  private var header: some View {
    SectionHeader(
      title: "Related exercises",
      caption: item.linkedExercises.isEmpty ? nil : "\(item.linkedExercises.count)",
      captionAccessibilityHidden: true,
      action: .init(
        title: editing ? "Done" : "Edit",
        accessibilityLabel: editing
          ? "Done editing related exercises" : "Edit related exercises",
        isDisabled: item.linkedExercises.isEmpty,
        perform: { editing.toggle() })
    )
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.top, IntradaSpacing.card)
    .padding(.bottom, item.linkedExercises.isEmpty ? 0 : IntradaSpacing.cardCompact)
  }

  @ViewBuilder private var rows: some View {
    if editing {
      ForEach(Array(item.linkedExercises.enumerated()), id: \.element.id) { index, exercise in
        if index > 0 {
          HairlineDivider()
        }
        LinkedExerciseEditRow(
          exercise: exercise,
          isFirst: index == 0,
          isLast: index == item.linkedExercises.count - 1,
          onMoveUp: { moveExercise(at: index, by: -1) },
          onMoveDown: { moveExercise(at: index, by: 1) },
          onRemove: { removeExercise(id: exercise.id) })
      }
    } else {
      ForEach(Array(item.linkedExercises.enumerated()), id: \.element.id) { index, exercise in
        if index > 0 {
          HairlineDivider()
        }
        NavigationLink(value: exercise.id) {
          LinkedExerciseRow(exercise: exercise)
        }
        .buttonStyle(.plain)
      }
    }
  }

  // The brand bar belongs to the empty state, where there is one obvious next
  // step; once the card holds exercises the action recedes (T18).
  private var populatedFooterActions: some View {
    Button(action: onAdd) {
      Label("Add exercise", systemImage: "plus")
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.accent)
        .frame(maxWidth: .infinity)
        .padding(.vertical, IntradaSpacing.cardCompact)
        .contentShape(Rectangle())
    }
    .buttonStyle(.plain)
    .accessibilityLabel("Add an exercise for this piece")
    .accessibilityIdentifier("libraryDetail.addExercise")
    .padding(.horizontal, IntradaSpacing.controlGap)
    .padding(.vertical, IntradaSpacing.controlGap)
  }

  // Drawn as a control, not a section: read as plain content its eyebrow gets
  // mistaken for the button, and it creates several library items (T18).
  @ViewBuilder private var suggestionsFromChart: some View {
    if let preview = item.scaffoldPreview {
      VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
        Eyebrow("From the chord chart")
        Button(action: onShowSuggestions) {
          HStack(spacing: IntradaSpacing.cardCompact) {
            Image(systemName: "sparkles")
              .font(IntradaFont.bodyMedium)
              .foregroundStyle(IntradaColor.exerciseBadgeFg)
              .accessibilityHidden(true)
            VStack(alignment: .leading, spacing: 3) {
              Text(suggestionHeadline(preview))
                .font(IntradaFont.bodyMedium)
                .foregroundStyle(IntradaColor.ink)
                .multilineTextAlignment(.leading)
              Text(suggestionSubtitle(preview))
                .font(IntradaFont.meta)
                .foregroundStyle(IntradaColor.inkSecondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            Image(systemName: "chevron.right")
              .font(IntradaFont.meta)
              .foregroundStyle(IntradaColor.exerciseBadgeFg)
              .accessibilityHidden(true)
          }
          .padding(IntradaSpacing.cardCompact)
          .background(
            IntradaColor.surfaceSunken, in: RoundedRectangle(cornerRadius: IntradaRadius.card)
          )
          .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel(suggestionAccessibilityLabel(preview))
        .accessibilityHint("Opens the exercises worked out from these changes")
      }
      .padding(.horizontal, IntradaSpacing.card)
      .padding(.top, IntradaSpacing.cardCompact)
      .padding(.bottom, IntradaSpacing.card)
    }
  }

  /// Names the first two suggestions rather than counting them: what they are
  /// is the reason to tap, and a bare count says nothing about the music.
  private func suggestionHeadline(_ preview: ScaffoldPreviewView) -> String {
    let names = preview.specs.filter { !$0.alreadyLinked }.map(\.title)
    guard let first = names.first else { return "All of them are already added" }
    let shown = names.prefix(2).enumerated().map { index, title in
      index == 0 ? title : lowercasingFirstLetter(title)
    }
    let rest = names.count - shown.count
    if rest > 0 {
      return shown.joined(separator: ", ") + " and \(rest) more"
    }
    return shown.count == 2 ? shown.joined(separator: " and ") : first
  }

  /// "Shells, guide-tone lines and 3 more": the run reads as one sentence, so
  /// every title after the first drops its capital (tone doc, sentence case).
  private func lowercasingFirstLetter(_ title: String) -> String {
    guard let first = title.first else { return title }
    return first.lowercased() + title.dropFirst()
  }

  private func suggestionSubtitle(_ preview: ScaffoldPreviewView) -> String {
    let added = preview.specs.filter(\.alreadyLinked).count
    if added > 0 {
      return "\(added) of \(preview.specs.count) already added"
    }
    return "Worked out from these changes, in \(preview.key)"
  }

  private func suggestionAccessibilityLabel(_ preview: ScaffoldPreviewView) -> String {
    "From the chord chart: \(suggestionHeadline(preview)) · \(suggestionSubtitle(preview))"
  }

  private func moveExercise(at index: Int, by delta: Int) {
    var ids = item.linkedExercises.map(\.id)
    let dest = index + delta
    guard dest >= 0, dest < ids.count else { return }
    ids.swapAt(index, dest)
    store.send(
      .item(.reorderLinkedExercises(pieceId: item.id, orderedIds: ids)), onSuccess: .selection)
  }

  private func removeExercise(id: String) {
    store.send(.item(.unlinkExercise(pieceId: item.id, exerciseId: id)), onSuccess: .impact)
  }
}

private struct LinkedExerciseTitle: View {
  let exercise: LinkedExerciseView

  var body: some View {
    // spacing: 3 is a tight title/meta baseline gap, below the token scale floor.
    VStack(alignment: .leading, spacing: 3) {
      Text(exercise.title)
        .font(IntradaFont.cardTitle())
        .foregroundStyle(IntradaColor.ink)
      if let meta = exercise.metaLine {
        Text(meta)
          .font(IntradaFont.meta)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
    }
    .frame(maxWidth: .infinity, alignment: .leading)
  }
}

/// Normal-mode row: exercise type bar + title + key/tempo meta + trailing score ring.
private struct LinkedExerciseRow: View {
  let exercise: LinkedExerciseView

  var body: some View {
    HStack(spacing: IntradaSpacing.card) {
      LinkedExerciseTitle(exercise: exercise)
      // The score *on this piece* (#1087 B2), not the exercise's flat overall;
      // the section caption tells the reader which. Unrated until practised here.
      ScoreRing(score: exercise.pieceContextScore.map(Int.init), size: 44)
    }
    .padding(.vertical, IntradaSpacing.card)
    .padding(.leading, 20)
    .padding(.trailing, IntradaSpacing.card)
    .background(IntradaColor.cardFill)
    .overlay(alignment: .leading) {
      ItemKind.exercise.bar.frame(width: 4)
    }
    .accessibilityElement(children: .combine)
    .accessibilityLabel(accessibilityLabel)
  }

  private var accessibilityLabel: String {
    var parts = ["Exercise", exercise.title]
    if let meta = exercise.metaSpoken { parts.append(meta) }
    if let score = exercise.pieceContextScore {
      parts.append("Mark \(score) of 10 on this piece")
    } else {
      parts.append("Not yet rated on this piece")
    }
    return parts.joined(separator: ", ")
  }
}

/// Edit-mode row: remove button + title + meta + up/down move buttons (VoiceOver-accessible reorder).
private struct LinkedExerciseEditRow: View {
  let exercise: LinkedExerciseView
  let isFirst: Bool
  let isLast: Bool
  let onMoveUp: () -> Void
  let onMoveDown: () -> Void
  let onRemove: () -> Void

  var body: some View {
    HStack(spacing: IntradaSpacing.cardCompact) {
      LinkedExerciseTitle(exercise: exercise)
      HStack(spacing: IntradaSpacing.controlGap) {
        VStack(spacing: 0) {
          Button(action: onMoveUp) {
            Image(systemName: "chevron.up")
              .imageScale(.small)
              .font(IntradaFont.meta)
              .foregroundStyle(isFirst ? IntradaColor.inkFainter : IntradaColor.inkSecondary)
          }
          .buttonStyle(.plain)
          .disabled(isFirst)
          .accessibilityLabel("Move \(exercise.title) up")
          Button(action: onMoveDown) {
            Image(systemName: "chevron.down")
              .imageScale(.small)
              .font(IntradaFont.meta)
              .foregroundStyle(isLast ? IntradaColor.inkFainter : IntradaColor.inkSecondary)
          }
          .buttonStyle(.plain)
          .disabled(isLast)
          .accessibilityLabel("Move \(exercise.title) down")
        }
        Button(action: onRemove) {
          Image(systemName: "minus.circle")
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.danger)
        }
        .buttonStyle(.plain)
        .accessibilityLabel("Remove \(exercise.title) from related exercises")
      }
    }
    .padding(.vertical, IntradaSpacing.cardCompact)
    .padding(.horizontal, IntradaSpacing.card)
    .background(IntradaColor.cardFill)
  }
}
