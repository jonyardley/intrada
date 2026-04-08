import SwiftUI

/// Focus-mode active session view.
///
/// Replaces the placeholder. Shows current item with timer, controls,
/// and optionally a rep counter. Hides nav/tab bars for focus mode.
/// Timer is shell-local (SwiftUI Timer publisher).
///
/// Named `ActivePracticeView` to avoid collision with the generated
/// `ActiveSessionView` struct from SharedTypes.
struct ActivePracticeView: View {
    @Environment(IntradaCore.self) private var core
    @Environment(\.horizontalSizeClass) private var sizeClass

    // MARK: - Shell-Local State

    @State private var elapsedSeconds: Int = 0
    @State private var isPaused: Bool = false
    @State private var showTransitionPrompt: Bool = false
    @State private var showPauseOverlay: Bool = false
    @State private var showEndEarlyConfirmation: Bool = false
    @State private var showAbandonConfirmation: Bool = false

    /// 1Hz timer that drives the elapsed counter.
    private let timer = Timer.publish(every: 1, on: .main, in: .common).autoconnect()

    // MARK: - Computed

    private var session: ActiveSessionView? {
        core.viewModel.activeSession
    }

    private var isLastItem: Bool {
        guard let session else { return false }
        return session.currentPosition + 1 >= session.totalItems
    }

    private var positionLabel: String {
        guard let session else { return "" }
        return "ITEM \(session.currentPosition + 1) OF \(session.totalItems)"
    }

    private var plannedDuration: UInt32? {
        session?.currentPlannedDurationSecs
    }

    private var ringProgress: Double {
        guard let planned = plannedDuration, planned > 0 else { return 0 }
        return min(Double(elapsedSeconds) / Double(planned), 1.0)
    }

    private var timerDisplay: String {
        if let planned = plannedDuration {
            let remaining = max(0, Int(planned) - elapsedSeconds)
            return formatTime(remaining)
        } else {
            return formatTime(elapsedSeconds)
        }
    }

    private var timerSubtitle: String? {
        guard let planned = plannedDuration else { return nil }
        return "of \(formatTime(Int(planned)))"
    }

    // MARK: - Body

    var body: some View {
        ZStack {
            Color.backgroundApp.ignoresSafeArea()

            if sizeClass == .regular {
                iPadLayout
            } else {
                iPhoneLayout
            }

            // Pause overlay (rendered on top)
            if showPauseOverlay {
                pauseOverlay
            }
        }
        .toolbar(.hidden, for: .navigationBar)
        .toolbar(.hidden, for: .tabBar)
        .onReceive(timer) { _ in
            guard !isPaused, !showTransitionPrompt else { return }
            elapsedSeconds += 1

            // Auto-trigger transition prompt when planned duration expires
            if let planned = plannedDuration,
               elapsedSeconds >= Int(planned),
               !showTransitionPrompt {
                showTransitionPrompt = true
            }
        }
        .onChange(of: session?.currentPosition) { _, _ in
            // Reset timer when item changes
            elapsedSeconds = 0
        }
        .sheet(isPresented: $showTransitionPrompt) {
            if let session {
                TransitionPromptSheet(
                    session: session,
                    isLastItem: isLastItem,
                    onContinue: { score, tempo, notes in
                        dispatchScoring(score: score, tempo: tempo, notes: notes)
                        advanceItem()
                    },
                    onSkip: {
                        advanceItem()
                    }
                )
                .presentationDetents([.medium, .large])
                .presentationDragIndicator(.visible)
            }
        }
        .confirmationDialog(
            "End this session?",
            isPresented: $showEndEarlyConfirmation,
            titleVisibility: .visible
        ) {
            Button("End Early") {
                let now = ISO8601DateFormatter().string(from: Date())
                core.update(.session(.endSessionEarly(now: now)))
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("Completed items will be saved.")
        }
        .confirmationDialog(
            "Discard this session?",
            isPresented: $showAbandonConfirmation,
            titleVisibility: .visible
        ) {
            Button("Abandon", role: .destructive) {
                core.update(.session(.abandonSession))
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("All progress will be lost.")
        }
    }

    // MARK: - iPhone Layout

    private var iPhoneLayout: some View {
        VStack(spacing: 0) {
            // Position label
            Text(positionLabel)
                .font(.system(size: 12, weight: .semibold))
                .tracking(1.5)
                .foregroundStyle(Color.textSecondary)
                .padding(.top, Spacing.card)

            Spacer()

            // Ring + timer area (ghosted)
            ringTimerArea(size: 160)

            Spacer()

            // Item info
            itemInfoSection

            Spacer()

            // Rep counter (conditional)
            if let session, session.currentRepTarget != nil {
                repCounterSection
                    .padding(.horizontal, Spacing.cardComfortable)

                Spacer()
            }

            // Controls
            controlsSection
                .padding(.horizontal, Spacing.cardComfortable)
                .padding(.bottom, Spacing.section)
        }
    }

    // MARK: - iPad Layout

    private var iPadLayout: some View {
        HStack(spacing: 0) {
            // Sidebar
            sessionSidebar
                .frame(width: 320)

            Divider()
                .background(Color.borderDefault)

            // Main focus area
            VStack(spacing: Spacing.cardComfortable) {
                Text(positionLabel)
                    .font(.system(size: 12, weight: .semibold))
                    .tracking(1.5)
                    .foregroundStyle(Color.textSecondary)

                ringTimerArea(size: 160)

                itemInfoSection

                if let session, session.currentRepTarget != nil {
                    repCounterSection
                        .frame(maxWidth: 400)
                }

                controlsSection
                    .frame(maxWidth: 400)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .padding(Spacing.cardComfortable)
        }
    }

    // MARK: - Ring + Timer

    @ViewBuilder
    private func ringTimerArea(size: CGFloat) -> some View {
        ZStack {
            if plannedDuration != nil {
                // Countdown mode: ghosted ring with remaining time
                ProgressRingView(
                    progress: ringProgress,
                    lineWidth: 5,
                    trackOpacity: 0.3,
                    fillOpacity: 0.25
                )
                .frame(width: size, height: size)
            }

            VStack(spacing: 2) {
                Text(timerDisplay)
                    .font(.system(size: 18, design: .monospaced))
                    .foregroundStyle(Color.textPrimary)

                if let subtitle = timerSubtitle {
                    Text(subtitle)
                        .font(.system(size: 10))
                        .foregroundStyle(Color.textFaint)
                }
            }
        }
        .frame(width: size, height: size)
    }

    // MARK: - Item Info

    private var itemInfoSection: some View {
        VStack(spacing: 8) {
            if let session {
                Text(session.currentItemTitle)
                    .font(.system(size: 24, weight: .bold))
                    .foregroundStyle(Color.textPrimary)
                    .multilineTextAlignment(.center)

                TypeBadge(kind: session.currentItemType)

                if let intention = currentEntryIntention {
                    Text(intention)
                        .font(.system(size: 14))
                        .foregroundStyle(Color.textSecondary)
                        .multilineTextAlignment(.center)
                }
            }
        }
        .padding(.horizontal, Spacing.cardComfortable)
    }

    private var currentEntryIntention: String? {
        guard let session else { return nil }
        let pos = Int(session.currentPosition)
        guard pos < session.entries.count else { return nil }
        return session.entries[pos].intention
    }

    // MARK: - Rep Counter

    private var repCounterSection: some View {
        Group {
            if let session,
               let target = session.currentRepTarget {
                RepCounterView(
                    count: session.currentRepCount ?? 0,
                    target: target,
                    targetReached: session.currentRepTargetReached ?? false,
                    onGotIt: { core.update(.session(.repGotIt)) },
                    onMissed: { core.update(.session(.repMissed)) }
                )
            }
        }
    }

    // MARK: - Controls

    private var controlsSection: some View {
        VStack(spacing: Spacing.cardCompact) {
            HStack(spacing: Spacing.cardCompact) {
                if isLastItem {
                    ButtonView("Finish", variant: .primary) {
                        showTransitionPrompt = true
                    }
                } else {
                    ButtonView("Next Item", variant: .primary) {
                        showTransitionPrompt = true
                    }
                }

                ButtonView("End Early", variant: .secondary) {
                    showEndEarlyConfirmation = true
                }
                .frame(width: 120)
            }

            Button {
                isPaused = true
                showPauseOverlay = true
            } label: {
                Image(systemName: "pause.circle")
                    .font(.system(size: 24))
                    .foregroundStyle(Color.textFaint)
            }
            .accessibilityLabel("Pause session")
        }
    }

    // MARK: - Pause Overlay

    private var pauseOverlay: some View {
        ZStack {
            Color.surfaceOverlay
                .ignoresSafeArea()
                .onTapGesture {
                    isPaused = false
                    showPauseOverlay = false
                }

            VStack(spacing: Spacing.card) {
                Image(systemName: "pause.circle")
                    .font(.system(size: 48))
                    .foregroundStyle(Color.accentText)

                Text("Session Paused")
                    .font(.system(size: 20, weight: .bold))
                    .foregroundStyle(Color.textPrimary)

                Text("\(positionLabel) · \(formatTime(elapsedSeconds)) elapsed")
                    .font(.system(size: 13))
                    .foregroundStyle(Color.textMuted)

                Divider()
                    .background(Color.borderDefault)
                    .padding(.vertical, 4)

                VStack(spacing: Spacing.cardCompact) {
                    ButtonView("Resume", variant: .primary) {
                        isPaused = false
                        showPauseOverlay = false
                    }

                    ButtonView("End Early", variant: .secondary) {
                        showPauseOverlay = false
                        isPaused = false
                        showEndEarlyConfirmation = true
                    }

                    ButtonView("Abandon Session", variant: .danger) {
                        showPauseOverlay = false
                        isPaused = false
                        showAbandonConfirmation = true
                    }
                }
            }
            .padding(Spacing.cardComfortable)
            .frame(width: 320)
            .background(Color.surfaceFallback)
            .clipShape(RoundedRectangle(cornerRadius: 16))
            .overlay(
                RoundedRectangle(cornerRadius: 16)
                    .stroke(Color.borderCard, lineWidth: 1)
            )
        }
    }

    // MARK: - iPad Sidebar

    private var sessionSidebar: some View {
        VStack(alignment: .leading, spacing: 0) {
            VStack(alignment: .leading, spacing: 4) {
                Text("Practice Session")
                    .font(.heading(size: 20))
                    .foregroundStyle(Color.textPrimary)

                if let intention = session?.sessionIntention {
                    Text(intention)
                        .font(.system(size: 13))
                        .foregroundStyle(Color.textMuted)
                }
            }
            .padding(.horizontal, Spacing.cardComfortable)
            .padding(.vertical, Spacing.card)

            Divider().background(Color.borderDefault)

            HStack(spacing: Spacing.card) {
                VStack(alignment: .leading, spacing: 2) {
                    Text("ELAPSED")
                        .font(.system(size: 9, weight: .semibold))
                        .tracking(1.5)
                        .foregroundStyle(Color.textFaint)
                    Text(formatTime(elapsedSeconds))
                        .font(.system(size: 16, weight: .semibold, design: .monospaced))
                        .foregroundStyle(Color.textPrimary)
                }
                .frame(maxWidth: .infinity, alignment: .leading)

                VStack(alignment: .leading, spacing: 2) {
                    Text("REMAINING")
                        .font(.system(size: 9, weight: .semibold))
                        .tracking(1.5)
                        .foregroundStyle(Color.textFaint)
                    Text(formatRemainingTime())
                        .font(.system(size: 16, weight: .semibold, design: .monospaced))
                        .foregroundStyle(Color.textPrimary)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }
            .padding(.horizontal, Spacing.cardComfortable)
            .padding(.vertical, Spacing.cardCompact)

            Divider().background(Color.borderDefault)

            Text("SETLIST")
                .font(.system(size: 9, weight: .semibold))
                .tracking(1.5)
                .foregroundStyle(Color.textFaint)
                .padding(.horizontal, Spacing.cardComfortable)
                .padding(.top, Spacing.cardCompact)
                .padding(.bottom, 8)

            ScrollView {
                LazyVStack(spacing: 0) {
                    if let session {
                        ForEach(Array(session.entries.enumerated()), id: \.element.id) { (index: Int, entry: SetlistEntryView) in
                            sidebarItemRow(entry: entry, index: index)
                        }
                    }
                }
            }

            Spacer()
        }
        .background(Color.backgroundApp)
    }

    @ViewBuilder
    private func sidebarItemRow(entry: SetlistEntryView, index: Int) -> some View {
        let isCurrent = UInt64(index) == session?.currentPosition
        let isCompleted = entry.status == .completed

        HStack(spacing: 12) {
            if isCompleted {
                Image(systemName: "checkmark.circle.fill")
                    .font(.system(size: 18))
                    .foregroundStyle(Color.successText)
            } else if isCurrent {
                Image(systemName: "disc")
                    .font(.system(size: 18))
                    .foregroundStyle(Color.accentText)
            } else {
                Image(systemName: "circle")
                    .font(.system(size: 18))
                    .foregroundStyle(Color.textFaint)
            }

            VStack(alignment: .leading, spacing: 2) {
                Text(entry.itemTitle)
                    .font(.system(size: 13, weight: isCurrent ? .semibold : .medium))
                    .foregroundStyle(isCurrent ? Color.textPrimary : (isCompleted ? Color.textMuted : Color.textSecondary))

                HStack(spacing: 6) {
                    Text(entry.itemType.displayText)
                        .font(.system(size: 11))
                        .foregroundStyle(Color.textFaint)

                    if isCurrent {
                        Text("·").foregroundStyle(Color.textMuted)
                        Text(timerDisplay)
                            .font(.system(size: 11, weight: .medium))
                            .foregroundStyle(Color.accentText)
                    } else if let planned = entry.plannedDurationDisplay {
                        Text("·").foregroundStyle(Color.textFaint)
                        Text(planned)
                            .font(.system(size: 11))
                            .foregroundStyle(Color.textFaint)
                    }

                    if isCompleted, let score = entry.score {
                        Text("·").foregroundStyle(Color.textFaint)
                        Text("★ \(score)")
                            .font(.system(size: 11, weight: .medium))
                            .foregroundStyle(Color.warmAccentText)
                    }
                }
            }
        }
        .padding(.horizontal, Spacing.cardComfortable)
        .padding(.vertical, 10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(isCurrent ? Color.surfaceSecondary : Color.clear)
        .overlay(alignment: .leading) {
            if isCurrent {
                Rectangle()
                    .fill(Color.accent)
                    .frame(width: 3)
            }
        }
    }

    // MARK: - Actions

    private func advanceItem() {
        let now = ISO8601DateFormatter().string(from: Date())
        if isLastItem {
            core.update(.session(.finishSession(now: now)))
        } else {
            core.update(.session(.nextItem(now: now)))
        }
        showTransitionPrompt = false
        elapsedSeconds = 0
    }

    private func dispatchScoring(score: UInt8?, tempo: UInt16?, notes: String?) {
        guard let session else { return }
        let pos = Int(session.currentPosition)
        guard pos < session.entries.count else { return }
        let entryId = session.entries[pos].id

        if let score {
            core.update(.session(.updateEntryScore(entryId: entryId, score: score)))
        }
        if let tempo {
            core.update(.session(.updateEntryTempo(entryId: entryId, tempo: tempo)))
        }
        if let notes, !notes.isEmpty {
            core.update(.session(.updateEntryNotes(entryId: entryId, notes: notes)))
        }
    }

    // MARK: - Helpers

    private func formatTime(_ seconds: Int) -> String {
        let mins = seconds / 60
        let secs = seconds % 60
        return String(format: "%d:%02d", mins, secs)
    }

    private func formatRemainingTime() -> String {
        guard let session else { return "0:00" }
        var totalPlanned = 0

        for (index, entry) in session.entries.enumerated() {
            if UInt64(index) < session.currentPosition {
                continue
            } else if UInt64(index) == session.currentPosition {
                if let planned = entry.plannedDurationSecs {
                    totalPlanned += max(0, Int(planned) - elapsedSeconds)
                }
            } else {
                if let planned = entry.plannedDurationSecs {
                    totalPlanned += Int(planned)
                }
            }
        }

        return formatTime(totalPlanned)
    }
}

#Preview("ActivePracticeView") {
    ActivePracticeView()
        .environment(IntradaCore())
        .preferredColorScheme(.dark)
}
