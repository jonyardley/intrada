import SharedTypes
import SwiftUI

/// Post-session review (the player's Summary). Renders the core's `SummaryView`:
/// review the items as they happened, adjust a rushed score, add a whole-session
/// note, then Save (persists + returns to Idle) or Discard. Reached after the
/// last item — per-item scores are already on the entries.
struct SessionSummaryScreen: View {
  @Environment(Store.self) private var store
  @State private var note = ""
  @State private var confirmingDiscard = false

  private var summary: SummaryView? { store.viewModel?.summary }

  var body: some View {
    ZStack {
      PaperBackground()
      if let summary {
        VStack(spacing: IntradaSpacing.card) {
          header(summary)
          ledger(summary)
          noteField
          controls
        }
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.top, 40)
        .padding(.bottom, IntradaSpacing.card)
      }
    }
    .onAppear { note = summary?.notes ?? "" }
    .alert("Discard this session?", isPresented: $confirmingDiscard) {
      Button("Discard", role: .destructive) { store.send(.session(.discardSession)) }
      Button("Keep", role: .cancel) {}
    } message: {
      Text("This practice won't be saved.")
    }
  }

  // ── Header ──

  private func header(_ summary: SummaryView) -> some View {
    VStack(alignment: .leading, spacing: 4) {
      Text(summary.completionStatus == .endedEarly ? "Ended early" : "Nice work")
        .font(IntradaFont.pageTitle(28))
        .foregroundStyle(IntradaColor.ink)
      Text(subtitle(summary))
        .font(IntradaFont.subtitle)
        .foregroundStyle(IntradaColor.inkSecondary)
    }
    .frame(maxWidth: .infinity, alignment: .leading)
  }

  private func subtitle(_ summary: SummaryView) -> String {
    let done = summary.entries.filter { $0.status == .completed }.count
    let total = summary.entries.count
    return "\(summary.totalDurationDisplay) · \(done) of \(total) done"
  }

  // ── Ledger ──

  private func ledger(_ summary: SummaryView) -> some View {
    VStack(alignment: .leading, spacing: 0) {
      Text("TODAY'S SESSION")
        .font(IntradaFont.badge)
        .tracking(1.5)
        .foregroundStyle(IntradaColor.inkFaint)
        .padding(.bottom, IntradaSpacing.controlGap)
      ScrollView {
        VStack(spacing: 0) {
          ForEach(Array(summary.entries.enumerated()), id: \.element.id) { index, entry in
            row(entry)
            if index < summary.entries.count - 1 {
              Rectangle().fill(IntradaColor.hairline).frame(height: 1)
            }
          }
        }
      }
      .scrollEdgeShadow()
    }
    .frame(maxHeight: .infinity)
  }

  private func row(_ entry: SetlistEntryView) -> some View {
    HStack(spacing: 11) {
      Circle().fill(entry.itemType.accent).frame(width: 8, height: 8)
      VStack(alignment: .leading, spacing: 2) {
        Text(entry.itemTitle)
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.ink)
        Text(metaLine(entry))
          .font(IntradaFont.micro)
          .foregroundStyle(IntradaColor.inkFaint)
      }
      Spacer()
      VStack(alignment: .trailing, spacing: 5) {
        Text(entry.durationDisplay)
          .font(IntradaFont.meta)
          .monospacedDigit()
          .foregroundStyle(IntradaColor.inkSecondary)
        if entry.status == .completed {
          scoreRow(entry)
        }
      }
    }
    .padding(.vertical, IntradaSpacing.cardCompact)
    .opacity(entry.status == .notAttempted ? 0.5 : 1)
  }

  private func metaLine(_ entry: SetlistEntryView) -> String {
    switch entry.status {
    case .skipped: return "Skipped"
    case .notAttempted: return "Not attempted"
    case .completed:
      var parts = [entry.itemType.label]
      if let tempo = entry.achievedTempo { parts.append("\(tempo) bpm") }
      return parts.joined(separator: " · ")
    }
  }

  /// Tappable 1–5 score. Tapping the current value clears it.
  private func scoreRow(_ entry: SetlistEntryView) -> some View {
    let score = entry.score.map(Int.init) ?? 0
    return HStack(spacing: 6) {
      ForEach(1...5, id: \.self) { value in
        Button {
          let next: UInt8? = entry.score == UInt8(value) ? nil : UInt8(value)
          store.send(.session(.updateEntryScore(entryId: entry.id, score: next)))
        } label: {
          Circle()
            .fill(score >= value ? AnyShapeStyle(IntradaColor.accent) : AnyShapeStyle(.clear))
            .frame(width: 18, height: 18)
            .overlay(
              Circle()
                .stroke(IntradaColor.divider, lineWidth: 1.5)
                .opacity(score >= value ? 0 : 1))
        }
        .buttonStyle(.plain)
      }
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel("Score for \(entry.itemTitle)")
    .accessibilityValue(score == 0 ? "not scored" : "\(score) of 5")
  }

  // ── Note + controls ──

  private var noteField: some View {
    TextField(
      "A note on the whole session…", text: $note, axis: .vertical
    )
    .lineLimit(2...4)
    .font(IntradaFont.field)
    .foregroundStyle(IntradaColor.ink)
    .padding(IntradaSpacing.cardCompact)
    .background(IntradaColor.cardFill)
    .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
    .overlay(
      RoundedRectangle(cornerRadius: IntradaRadius.card)
        .stroke(IntradaColor.hairline, lineWidth: 1)
    )
    .onChange(of: note) { _, value in
      // Push real edits only, and only while still in Summary — avoids an
      // identical round-trip on the seeded value and a "not in summary" core
      // error if the field settles during teardown.
      guard summary != nil else { return }
      let trimmed = value.isEmpty ? nil : value
      guard trimmed != summary?.notes else { return }
      store.send(.session(.updateSessionNotes(notes: trimmed)))
    }
  }

  private var controls: some View {
    VStack(spacing: 10) {
      Button {
        store.send(.session(.saveSession(now: SessionClock.nowRFC3339())))
      } label: {
        Text("Save session")
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.onAccent)
          .frame(maxWidth: .infinity)
          .padding(.vertical, IntradaSpacing.row)
          .background(LinearGradient.brandBar)
          .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
      }
      .buttonStyle(.plain)
      Button("Discard") { confirmingDiscard = true }
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.inkSecondary)
    }
  }
}

#if DEBUG
  #Preview("Completed") {
    SessionSummaryScreen().environment(Store.previewSummary)
  }

  #Preview("Ended early") {
    SessionSummaryScreen().environment(Store.previewSummaryEndedEarly)
  }
#endif
