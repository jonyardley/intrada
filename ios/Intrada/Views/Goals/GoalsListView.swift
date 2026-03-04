import SwiftUI

/// List of goals with status filtering and progress display.
struct GoalsListView: View {
    @Environment(IntradaCore.self) private var core

    @State private var filter: GoalFilter = .active
    @State private var showAddGoal = false

    private var goals: [GoalView] {
        let allGoals = core.viewModel.goals
        switch filter {
        case .active:
            return allGoals.filter { $0.status == "active" }
        case .completed:
            return allGoals.filter { $0.status == "completed" }
        case .all:
            return allGoals
        }
    }

    enum GoalFilter: String, CaseIterable {
        case active = "Active"
        case completed = "Completed"
        case all = "All"
    }

    var body: some View {
        ScrollView {
            VStack(spacing: 16) {
                // Filter picker
                Picker("Filter", selection: $filter) {
                    ForEach(GoalFilter.allCases, id: \.self) { f in
                        Text(f.rawValue).tag(f)
                    }
                }
                .pickerStyle(.segmented)
                .padding(.horizontal)

                if goals.isEmpty {
                    EmptyStateView(
                        icon: "target",
                        title: filter == .active ? "No active goals" : "No goals found",
                        message: filter == .active
                            ? "Set goals to track your practice progress."
                            : "Try a different filter.",
                        actionTitle: filter == .active ? "Add Goal" : nil,
                        action: filter == .active ? { showAddGoal = true } : nil
                    )
                } else {
                    LazyVStack(spacing: 12) {
                        ForEach(goals, id: \.id) { goal in
                            GoalCard(goal: goal)
                        }
                    }
                    .padding(.horizontal)
                }
            }
            .padding(.vertical)
        }
        .navigationTitle("Goals")
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button {
                    showAddGoal = true
                } label: {
                    Image(systemName: "plus")
                }
            }
        }
        .sheet(isPresented: $showAddGoal) {
            NavigationStack {
                GoalFormView()
            }
        }
    }
}

// MARK: - Goal Card

private struct GoalCard: View {
    let goal: GoalView
    @Environment(IntradaCore.self) private var core
    @State private var showActions = false

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            // Header
            HStack {
                Image(systemName: iconForKind(goal.kindType))
                    .foregroundStyle(.indigo)
                Text(goal.title)
                    .font(.headline)
                    .lineLimit(2)
                Spacer()

                Menu {
                    if goal.status == "active" {
                        Button {
                            core.update(.goal(.complete(id: goal.id)))
                        } label: {
                            Label("Mark Complete", systemImage: "checkmark.circle")
                        }
                        Button {
                            core.update(.goal(.archive(id: goal.id)))
                        } label: {
                            Label("Archive", systemImage: "archivebox")
                        }
                    }
                    if goal.status == "completed" || goal.status == "archived" {
                        Button {
                            core.update(.goal(.reactivate(id: goal.id)))
                        } label: {
                            Label("Reactivate", systemImage: "arrow.uturn.backward")
                        }
                    }
                    Button(role: .destructive) {
                        core.update(.goal(.delete(id: goal.id)))
                    } label: {
                        Label("Delete", systemImage: "trash")
                    }
                } label: {
                    Image(systemName: "ellipsis.circle")
                        .foregroundStyle(.secondary)
                }
            }

            // Kind label
            Text(goal.kindLabel)
                .font(.caption)
                .foregroundStyle(.secondary)

            // Item link (if applicable)
            if let itemTitle = goal.itemTitle {
                Label(itemTitle, systemImage: "music.note")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            // Progress
            if let progress = goal.progress {
                VStack(alignment: .leading, spacing: 4) {
                    HStack {
                        Text(progress.displayText)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                        Spacer()
                        Text("\(Int(progress.percentage))%")
                            .font(.caption)
                            .fontWeight(.medium)
                            .foregroundStyle(.indigo)
                    }

                    ProgressView(value: progress.percentage, total: 100)
                        .tint(progressColor(progress.percentage))
                }
            }

            // Status badge + deadline
            HStack {
                StatusBadge(status: goal.status)

                if let deadline = goal.deadline, !deadline.isEmpty {
                    Label(formatDeadline(deadline), systemImage: "calendar")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
            }
        }
        .padding()
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }

    private func iconForKind(_ kindType: String) -> String {
        switch kindType {
        case "session_frequency": return "calendar.badge.clock"
        case "practice_time": return "clock"
        case "item_mastery": return "star"
        case "milestone": return "flag"
        default: return "target"
        }
    }

    private func progressColor(_ percentage: Double) -> Color {
        if percentage >= 100 { return .green }
        if percentage >= 50 { return .indigo }
        return .orange
    }

    private func formatDeadline(_ isoString: String) -> String {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        var date = formatter.date(from: isoString)
        if date == nil {
            formatter.formatOptions = [.withInternetDateTime]
            date = formatter.date(from: isoString)
        }
        guard let d = date else { return isoString }
        let display = DateFormatter()
        display.dateStyle = .medium
        return display.string(from: d)
    }
}

// MARK: - Status Badge

private struct StatusBadge: View {
    let status: String

    var body: some View {
        Text(status.capitalized)
            .font(.caption2)
            .fontWeight(.medium)
            .padding(.horizontal, 8)
            .padding(.vertical, 3)
            .background(backgroundColor)
            .foregroundStyle(foregroundColor)
            .clipShape(Capsule())
    }

    private var backgroundColor: Color {
        switch status {
        case "active": return .indigo.opacity(0.15)
        case "completed": return .green.opacity(0.15)
        case "archived": return .gray.opacity(0.15)
        default: return .gray.opacity(0.15)
        }
    }

    private var foregroundColor: Color {
        switch status {
        case "active": return .indigo
        case "completed": return .green
        case "archived": return .gray
        default: return .gray
        }
    }
}
