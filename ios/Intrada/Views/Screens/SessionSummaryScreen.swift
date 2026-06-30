import SharedTypes
import SwiftUI

/// Post-session review (the player's Summary) — the celebration beat. Renders the
/// core's `SummaryView`: confetti + headline, an optional mastery toast, the recap
/// of what was played (per-item 1–10 scores), a whole-session note, an overall
/// session score (1–10), then Save (persists + returns to Idle) or Discard.
/// Reached after the last item.
struct SessionSummaryScreen: View {
  @Environment(Store.self) private var store
  @State private var note = ""
  @State private var confirmingDiscard = false

  private var summary: SummaryView? { store.viewModel?.summary }

  var body: some View {
    ZStack {
      PaperBackground()
      if let summary {
        ScrollView {
          VStack(alignment: .leading, spacing: IntradaSpacing.section) {
            headline(summary).fadeUp(0)
            if let toast = topMover(summary) {
              MasteryDeltaToast(
                title: "\(toast.itemTitle) moved up",
                subtitle: nil,
                was: Int(toast.previousScore ?? 0),
                now: Int(toast.currentScore)
              )
              .fadeUp(1)
            }
            recap(summary).fadeUp(2)
            noteSection.fadeUp(3)
            sessionScoreRow(summary).fadeUp(4)
            controls.fadeUp(5)
          }
          .padding(.horizontal, IntradaSpacing.card)
          .padding(.top, 40)
          .padding(.bottom, IntradaSpacing.card)
        }
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

  // ── Headline ──

  private func headline(_ summary: SummaryView) -> some View {
    ZStack(alignment: .topLeading) {
      Confetti()
        .frame(maxWidth: .infinity, alignment: .center)
        .allowsHitTesting(false)
      VStack(alignment: .leading, spacing: 4) {
        Eyebrow("Session complete", tint: IntradaColor.exerciseBadgeFg)
        Text(summary.completionStatus == .endedEarly ? "Ended early." : "Nice work.")
          .font(IntradaFont.pageTitle(34))
          .foregroundStyle(IntradaColor.ink)
        Text(headlineSubtitle(summary))
          .font(IntradaFont.subtitle)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
    }
    .frame(maxWidth: .infinity, alignment: .leading)
  }

  private func headlineSubtitle(_ summary: SummaryView) -> String {
    let done = summary.entries.filter { $0.status == .completed }.count
    return "\(summary.totalDurationDisplay) · \(done) of \(summary.entries.count)"
  }

  private func topMover(_ summary: SummaryView) -> ScoreChange? {
    guard let changes = store.viewModel?.analytics?.scoreChanges else { return nil }
    let titles = Swift.Set(summary.entries.map(\.itemTitle))
    return changes
      .filter { titles.contains($0.itemTitle) && $0.delta > 0 }
      .max { $0.delta < $1.delta }
  }

  // ── Recap ──

  private func recap(_ summary: SummaryView) -> some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      Eyebrow("What you played")
      VStack(spacing: 0) {
        ForEach(Array(summary.entries.enumerated()), id: \.element.id) { index, entry in
          row(entry)
          if index < summary.entries.count - 1 {
            Rectangle().fill(IntradaColor.hairline).frame(height: 1)
          }
        }
      }
    }
  }

  private func row(_ entry: SetlistEntryView) -> some View {
    let unfinished = entry.status == .notAttempted
    return VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
      HStack(spacing: 11) {
        dot(entry, unfinished: unfinished)
        VStack(alignment: .leading, spacing: 2) {
          Text(entry.itemTitle)
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.ink)
          Text(metaLine(entry, unfinished: unfinished))
            .font(IntradaFont.micro)
            .foregroundStyle(IntradaColor.inkFaint)
        }
        Spacer()
        if !unfinished {
          Text(entry.durationDisplay)
            .font(IntradaFont.meta)
            .monospacedDigit()
            .foregroundStyle(IntradaColor.inkSecondary)
        }
      }
      if entry.status == .completed {
        scoreRow(entry)
      }
    }
    .padding(.vertical, IntradaSpacing.cardCompact)
    .opacity(unfinished ? 0.5 : 1)
  }

  @ViewBuilder
  private func dot(_ entry: SetlistEntryView, unfinished: Bool) -> some View {
    if unfinished {
      Circle()
        .strokeBorder(IntradaColor.figureMuted, lineWidth: 1.5)
        .frame(width: 8, height: 8)
    } else {
      Circle().fill(entry.itemType.accent).frame(width: 8, height: 8)
    }
  }

  private func metaLine(_ entry: SetlistEntryView, unfinished: Bool) -> String {
    if unfinished { return "Saved for next time — no pressure" }
    var parts = [entry.itemType.label]
    if let tempo = entry.achievedTempo { parts.append("\(tempo) bpm") }
    return parts.joined(separator: " · ")
  }

  private func scoreRow(_ entry: SetlistEntryView) -> some View {
    ScoreSelector(
      score: entry.score.map(Int.init) ?? 0,
      accessibilityLabel: "Score for \(entry.itemTitle)"
    ) { next in
      store.send(.session(.updateEntryScore(entryId: entry.id, score: next)))
    }
    .padding(.leading, 19)
  }

  private func sessionScoreRow(_ summary: SummaryView) -> some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
      Text("Overall")
        .font(IntradaFont.metaMedium)
        .foregroundStyle(IntradaColor.inkSecondary)
      ScoreSelector(
        score: summary.sessionScore.map(Int.init) ?? 0,
        accessibilityLabel: "Overall session score"
      ) { next in
        store.send(.session(.updateSessionScore(score: next)))
      }
    }
  }

  // ── Note + controls ──

  private var noteSection: some View {
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
      .buttonStyle(PressRebound())
      Button("Discard") { confirmingDiscard = true }
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.inkSecondary)
    }
  }

  // ── Confetti ──

  /// A one-shot burst behind the headline. Renders nothing under Reduce Motion or
  /// in UI tests, so the celebration is deterministic and motion-safe.
  private struct Confetti: View {
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @Environment(\.intradaMotionDisabled) private var motionDisabled
    @State private var fallen = false

    private struct Piece: Identifiable {
      let id = UUID()
      let x: CGFloat
      let size: CGFloat
      let color: Color
      let isCircle: Bool
      let delay: Double
    }

    private static let pieces: [Piece] = {
      let colors = [
        IntradaColor.exerciseAccent, IntradaColor.accent, IntradaColor.brandGradientStart,
      ]
      let xs: [CGFloat] = [-110, -64, -22, 24, 70, 112]
      return xs.enumerated().map { index, x in
        Piece(
          x: x,
          size: index.isMultiple(of: 2) ? 7 : 8,
          color: colors[index % colors.count],
          isCircle: index.isMultiple(of: 3),
          delay: Double(index) * 0.05)
      }
    }()

    var body: some View {
      if reduceMotion || motionDisabled || UITestFlags.animationsDisabled {
        EmptyView()
      } else {
        ZStack {
          ForEach(Self.pieces) { piece in
            shape(piece)
              .foregroundStyle(piece.color)
              .frame(width: piece.size, height: piece.size)
              .offset(x: piece.x, y: fallen ? 96 : -24)
              .opacity(fallen ? 0 : 1)
          }
        }
        .frame(height: 96)
        .onAppear {
          withAnimation(.easeOut(duration: 1.3)) { fallen = true }
        }
      }
    }

    @ViewBuilder
    private func shape(_ piece: Piece) -> some View {
      if piece.isCircle {
        Circle()
      } else {
        RoundedRectangle(cornerRadius: 1.5)
      }
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
