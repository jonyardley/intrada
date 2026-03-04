import SwiftUI

/// Practice history with weekly calendar strip navigator.
struct SessionsListView: View {
    @Environment(IntradaCore.self) private var core

    @State private var weekOffset = 0
    @State private var selectedDate: Date?

    private var calendar: Calendar { Calendar.current }

    /// Start of the displayed week (Monday).
    private var weekStart: Date {
        let today = calendar.startOfDay(for: Date())
        let shifted = calendar.date(byAdding: .weekOfYear, value: weekOffset, to: today)!
        // Find Monday of that week
        let weekday = calendar.component(.weekday, from: shifted)
        let daysFromMonday = (weekday + 5) % 7 // Monday=0, Sun=6
        return calendar.date(byAdding: .day, value: -daysFromMonday, to: shifted)!
    }

    /// Days Mon–Sun for the current week.
    private var weekDays: [Date] {
        (0..<7).compactMap { calendar.date(byAdding: .day, value: $0, to: weekStart) }
    }

    /// Sessions grouped by date string "yyyy-MM-dd".
    private var sessionsByDate: [String: [PracticeSessionView]] {
        Dictionary(grouping: core.viewModel.sessions) { session in
            String(session.startedAt.prefix(10)) // "2026-03-04"
        }
    }

    /// Whether a given date has sessions.
    private func hasSessions(on date: Date) -> Bool {
        let key = dateKey(date)
        return sessionsByDate[key] != nil
    }

    /// Sessions for the selected date, sorted newest first.
    private var selectedSessions: [PracticeSessionView] {
        guard let date = selectedDate else { return [] }
        let key = dateKey(date)
        return (sessionsByDate[key] ?? []).sorted { $0.startedAt > $1.startedAt }
    }

    private func dateKey(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd"
        formatter.timeZone = TimeZone.current
        return formatter.string(from: date)
    }

    var body: some View {
        ScrollView {
            VStack(spacing: 16) {
                // Week Strip
                weekStripView

                // Session cards or empty state
                if selectedDate != nil {
                    if selectedSessions.isEmpty {
                        EmptyStateView(
                            icon: "calendar",
                            title: "No sessions",
                            message: "No practice sessions on this day."
                        )
                    } else {
                        LazyVStack(spacing: 12) {
                            ForEach(selectedSessions, id: \.id) { session in
                                NavigationLink(value: session.id) {
                                    SessionCardView(session: session)
                                }
                                .buttonStyle(.plain)
                            }
                        }
                        .padding(.horizontal)
                    }
                }

                // "Show all sessions" link
                NavigationLink("Show all sessions", value: "all-sessions")
                    .font(.subheadline)
                    .padding(.top, 8)
            }
            .padding(.vertical)
        }
        .navigationTitle("Practice")
        .navigationDestination(for: String.self) { id in
            if id == "all-sessions" {
                AllSessionsView()
            } else {
                SessionDetailView(sessionId: id)
            }
        }
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                NavigationLink {
                    NewSessionView()
                } label: {
                    Image(systemName: "plus")
                }
            }
        }
        .onAppear { autoSelectDate() }
        .onChange(of: weekOffset) { autoSelectDate() }
    }

    // MARK: - Week Strip

    private var weekStripView: some View {
        VStack(spacing: 8) {
            // Week navigation header
            HStack {
                Button {
                    weekOffset -= 1
                } label: {
                    Image(systemName: "chevron.left")
                        .font(.body.weight(.medium))
                }

                Spacer()

                Text(weekLabel)
                    .font(.subheadline)
                    .fontWeight(.medium)

                Spacer()

                if weekOffset != 0 {
                    Button("Today") {
                        weekOffset = 0
                    }
                    .font(.caption)
                }

                Button {
                    weekOffset += 1
                } label: {
                    Image(systemName: "chevron.right")
                        .font(.body.weight(.medium))
                }
            }
            .padding(.horizontal)

            // Day buttons
            HStack(spacing: 0) {
                ForEach(weekDays, id: \.self) { day in
                    DayButton(
                        date: day,
                        isSelected: selectedDate.map { calendar.isDate($0, inSameDayAs: day) } ?? false,
                        hasSessions: hasSessions(on: day)
                    ) {
                        selectedDate = day
                    }
                }
            }
            .padding(.horizontal, 4)
        }
    }

    private var weekLabel: String {
        let formatter = DateFormatter()
        formatter.dateFormat = "MMM d"
        let start = formatter.string(from: weekStart)
        let end = formatter.string(from: weekDays.last ?? weekStart)
        return "\(start) – \(end)"
    }

    /// Auto-select today, or the most recent day with sessions.
    private func autoSelectDate() {
        let today = calendar.startOfDay(for: Date())

        // If today is in this week and has sessions, select it
        if weekDays.contains(where: { calendar.isDate($0, inSameDayAs: today) }) && hasSessions(on: today) {
            selectedDate = today
            return
        }

        // Otherwise, find the most recent day in this week with sessions
        let recentDay = weekDays.reversed().first(where: { hasSessions(on: $0) })
        selectedDate = recentDay ?? (weekDays.contains(where: { calendar.isDate($0, inSameDayAs: today) }) ? today : weekDays.first)
    }
}

// MARK: - Day Button

private struct DayButton: View {
    let date: Date
    let isSelected: Bool
    let hasSessions: Bool
    let action: () -> Void

    private var calendar: Calendar { Calendar.current }

    var body: some View {
        Button(action: action) {
            VStack(spacing: 4) {
                Text(dayOfWeekLabel)
                    .font(.caption2)
                    .foregroundStyle(.secondary)

                Text("\(calendar.component(.day, from: date))")
                    .font(.callout)
                    .fontWeight(isSelected ? .bold : .regular)
                    .foregroundStyle(isSelected ? .white : .primary)
                    .frame(width: 36, height: 36)
                    .background(isSelected ? Color.indigo : Color.clear)
                    .clipShape(Circle())

                Circle()
                    .fill(hasSessions ? Color.indigo : Color.clear)
                    .frame(width: 5, height: 5)
            }
        }
        .buttonStyle(.plain)
        .frame(maxWidth: .infinity)
    }

    private var dayOfWeekLabel: String {
        let formatter = DateFormatter()
        formatter.dateFormat = "EEE"
        return formatter.string(from: date).prefix(3).uppercased()
    }
}

// MARK: - Session Card

struct SessionCardView: View {
    let session: PracticeSessionView

    @Environment(IntradaCore.self) private var core
    @State private var showDeleteConfirm = false

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text(formattedTime)
                        .font(.subheadline)
                        .fontWeight(.medium)

                    Text(session.totalDurationDisplay)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                Spacer()

                Text(statusLabel)
                    .font(.caption2)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 3)
                    .background(statusColor.opacity(0.15))
                    .foregroundStyle(statusColor)
                    .clipShape(Capsule())
            }

            if !session.entries.isEmpty {
                VStack(alignment: .leading, spacing: 2) {
                    ForEach(session.entries.prefix(3), id: \.id) { entry in
                        HStack(spacing: 6) {
                            TypeBadge(itemType: entry.itemType)
                            Text(entry.itemTitle)
                                .font(.caption)
                                .lineLimit(1)
                            Spacer()
                            if let score = entry.score {
                                ScoreIndicator(score: score)
                            }
                        }
                    }
                    if session.entries.count > 3 {
                        Text("+\(session.entries.count - 3) more")
                            .font(.caption2)
                            .foregroundStyle(.tertiary)
                    }
                }
            }
        }
        .padding()
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 12))
        .contextMenu {
            Button(role: .destructive) {
                showDeleteConfirm = true
            } label: {
                Label("Delete", systemImage: "trash")
            }
        }
        .deleteConfirmation(
            "Delete session?",
            message: "This session will be permanently removed.",
            isPresented: $showDeleteConfirm
        ) {
            core.update(.session(.discardSession))
        }
    }

    private var formattedTime: String {
        // startedAt is an ISO string like "2026-03-04T15:30:00Z"
        let dateStr = session.startedAt
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        var date = formatter.date(from: dateStr)
        if date == nil {
            formatter.formatOptions = [.withInternetDateTime]
            date = formatter.date(from: dateStr)
        }
        guard let d = date else { return dateStr }
        let display = DateFormatter()
        display.dateStyle = .none
        display.timeStyle = .short
        return display.string(from: d)
    }

    private var statusLabel: String {
        switch session.completionStatus {
        case "completed": "Completed"
        case "ended_early": "Ended Early"
        case "abandoned": "Abandoned"
        default: session.completionStatus.capitalized
        }
    }

    private var statusColor: Color {
        switch session.completionStatus {
        case "completed": .green
        case "ended_early": .orange
        case "abandoned": .red
        default: .secondary
        }
    }
}
