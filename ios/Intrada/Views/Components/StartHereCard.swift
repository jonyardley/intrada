import SharedTypes
import SwiftUI

/// The four first-session steps under the Practice hero (#2118). The ticks are
/// the core's, read from the library and the history, so the card never
/// disagrees with them; it goes after the first marked session.
struct StartHereCard: View {
  let progress: FirstRunView
  let onAdd: () -> Void
  let onStartSession: () -> Void

  @Environment(\.marker) private var marker
  @ScaledMetric(relativeTo: .body) private var tickSize: CGFloat = 24

  private struct Step {
    let title: String
    let done: Bool
    let action: () -> Void
  }

  private var steps: [Step] {
    [
      Step(
        title: progress.firstItemTitle.map { "Added \($0)" } ?? "Add a piece or exercise",
        done: progress.added, action: onAdd),
      Step(title: "Build a session", done: progress.built, action: onStartSession),
      Step(title: "Play it through", done: progress.played, action: onStartSession),
      Step(title: "Mark how it went", done: progress.marked, action: onStartSession),
    ]
  }

  var body: some View {
    let steps = steps
    let current = steps.firstIndex { !$0.done }
    VStack(alignment: .leading, spacing: 0) {
      let count = Text("\(steps.filter(\.done).count) of \(steps.count)")
        .font(IntradaFont.meta)
        .foregroundStyle(IntradaColor.inkSecondary)
      ViewThatFits(in: .horizontal) {
        HStack(alignment: .firstTextBaseline) {
          Eyebrow("Start here")
          Spacer()
          count
        }
        VStack(alignment: .leading, spacing: 4) {
          Eyebrow("Start here")
          count
        }
      }
      .padding(.horizontal, IntradaSpacing.card)
      .padding(.top, IntradaSpacing.card)
      .padding(.bottom, IntradaSpacing.controlGap)
      .accessibilityElement(children: .combine)
      ForEach(Array(steps.enumerated()), id: \.offset) { index, step in
        if index > 0 { HairlineDivider() }
        row(step, state: step.done ? .done : index == current ? .current : .later)
      }
    }
    .cardSurface()
    .accessibilityIdentifier("practice.startHere")
  }

  private enum RowState { case done, current, later }

  @ViewBuilder private func row(_ step: Step, state: RowState) -> some View {
    let label = HStack(spacing: IntradaSpacing.cardCompact) {
      tick(state)
      Text(step.title)
        .font(state == .current ? IntradaFont.bodyMedium : IntradaFont.body)
        .foregroundStyle(state == .later ? IntradaColor.inkSecondary : IntradaColor.ink)
        .fixedSize(horizontal: false, vertical: true)
        .frame(maxWidth: .infinity, alignment: .leading)
      if state == .current {
        Image(systemName: "chevron.right")
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.inkFaintIcon)
          .accessibilityHidden(true)
      }
    }
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.vertical, IntradaSpacing.cardCompact)
    .frame(minHeight: 44)
    .contentShape(Rectangle())

    if state == .current {
      Button(action: step.action) { label }
        .buttonStyle(.plain)
        .accessibilityElement(children: .ignore)
        .accessibilityLabel(step.title)
        .accessibilityHint("Next step")
        .accessibilityAddTraits(.isButton)
        .accessibilityIdentifier("practice.startHere.next")
    } else {
      label
        .accessibilityElement(children: .ignore)
        .accessibilityLabel(state == .done ? "\(step.title), done" : step.title)
    }
  }

  @ViewBuilder private func tick(_ state: RowState) -> some View {
    let size = min(tickSize, 36)
    switch state {
    case .done:
      Image(systemName: "checkmark")
        .iconSize(.caption, weight: .bold)
        .foregroundStyle(IntradaColor.onMarker)
        .frame(width: size, height: size)
        .background(marker, in: Circle())
    case .current:
      Circle()
        .strokeBorder(IntradaColor.ink, lineWidth: 1.5)
        .frame(width: size, height: size)
    case .later:
      Circle()
        .strokeBorder(IntradaColor.divider, lineWidth: 1.5)
        .frame(width: size, height: size)
    }
  }
}

#if DEBUG
  #Preview {
    StartHereCard(
      progress: FirstRunView(
        showsWelcome: false, showsStartHere: true, added: true, built: false, played: false,
        marked: false, firstItemTitle: "Clair de Lune"),
      onAdd: {}, onStartSession: {}
    )
    .padding()
  }
#endif
