import SwiftUI

/// Create a new goal with one of four types.
struct GoalFormView: View {
    @Environment(IntradaCore.self) private var core
    @Environment(\.dismiss) private var dismiss

    @State private var title = ""
    @State private var selectedKind: GoalKindOption = .sessionFrequency
    @State private var hasDeadline = false
    @State private var deadline = Date().addingTimeInterval(7 * 24 * 3600) // 1 week from now

    // Kind-specific fields
    @State private var targetDaysPerWeek: Int = 3
    @State private var targetMinutesPerWeek: Int = 120
    @State private var selectedItemId: String?
    @State private var targetScore: Int = 4
    @State private var milestoneDescription = ""

    @State private var showItemPicker = false

    private var items: [LibraryItemView] { core.viewModel.items }

    enum GoalKindOption: String, CaseIterable {
        case sessionFrequency = "Session Frequency"
        case practiceTime = "Practice Time"
        case itemMastery = "Item Mastery"
        case milestone = "Milestone"

        var icon: String {
            switch self {
            case .sessionFrequency: return "calendar.badge.clock"
            case .practiceTime: return "clock"
            case .itemMastery: return "star"
            case .milestone: return "flag"
            }
        }

        var description: String {
            switch self {
            case .sessionFrequency: return "Practice a certain number of days per week"
            case .practiceTime: return "Accumulate practice minutes per week"
            case .itemMastery: return "Reach a target score on a specific item"
            case .milestone: return "Achieve a specific practice milestone"
            }
        }
    }

    private var isValid: Bool {
        guard !title.isEmpty else { return false }
        switch selectedKind {
        case .sessionFrequency:
            return targetDaysPerWeek >= 1 && targetDaysPerWeek <= 7
        case .practiceTime:
            return targetMinutesPerWeek > 0
        case .itemMastery:
            return selectedItemId != nil && targetScore >= 1 && targetScore <= 5
        case .milestone:
            return !milestoneDescription.isEmpty
        }
    }

    var body: some View {
        Form {
            // Title
            Section("Goal") {
                TextField("What do you want to achieve?", text: $title)
            }

            // Kind picker
            Section("Type") {
                ForEach(GoalKindOption.allCases, id: \.self) { kind in
                    Button {
                        selectedKind = kind
                    } label: {
                        HStack(spacing: 12) {
                            Image(systemName: kind.icon)
                                .font(.title3)
                                .foregroundStyle(selectedKind == kind ? .indigo : .secondary)
                                .frame(width: 28)

                            VStack(alignment: .leading, spacing: 2) {
                                Text(kind.rawValue)
                                    .font(.subheadline)
                                    .fontWeight(.medium)
                                    .foregroundStyle(.primary)
                                Text(kind.description)
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                            }

                            Spacer()

                            if selectedKind == kind {
                                Image(systemName: "checkmark.circle.fill")
                                    .foregroundStyle(.indigo)
                            }
                        }
                    }
                    .buttonStyle(.plain)
                }
            }

            // Kind-specific configuration
            Section("Target") {
                switch selectedKind {
                case .sessionFrequency:
                    Stepper(
                        "\(targetDaysPerWeek) day\(targetDaysPerWeek == 1 ? "" : "s") per week",
                        value: $targetDaysPerWeek,
                        in: 1...7
                    )

                case .practiceTime:
                    Stepper(
                        "\(targetMinutesPerWeek) minutes per week",
                        value: $targetMinutesPerWeek,
                        in: 10...600,
                        step: 10
                    )

                case .itemMastery:
                    // Item selection
                    Button {
                        showItemPicker = true
                    } label: {
                        HStack {
                            Text("Item")
                                .foregroundStyle(.primary)
                            Spacer()
                            if let itemId = selectedItemId,
                               let item = items.first(where: { $0.id == itemId }) {
                                Text(item.title)
                                    .foregroundStyle(.secondary)
                            } else {
                                Text("Select an item")
                                    .foregroundStyle(.tertiary)
                            }
                            Image(systemName: "chevron.right")
                                .font(.caption)
                                .foregroundStyle(.tertiary)
                        }
                    }
                    .buttonStyle(.plain)

                    // Target score
                    HStack {
                        Text("Target Score")
                        Spacer()
                        HStack(spacing: 6) {
                            ForEach(1...5, id: \.self) { value in
                                Button {
                                    targetScore = value
                                } label: {
                                    Circle()
                                        .fill(value <= targetScore ? Color.indigo : Color.secondary.opacity(0.2))
                                        .frame(width: 28, height: 28)
                                        .overlay {
                                            Text("\(value)")
                                                .font(.caption)
                                                .fontWeight(.medium)
                                                .foregroundStyle(value <= targetScore ? .white : .secondary)
                                        }
                                }
                                .buttonStyle(.plain)
                            }
                        }
                    }

                case .milestone:
                    TextField("Describe the milestone", text: $milestoneDescription)
                }
            }

            // Deadline
            Section("Deadline") {
                Toggle("Set a deadline", isOn: $hasDeadline)

                if hasDeadline {
                    DatePicker(
                        "Due date",
                        selection: $deadline,
                        in: Date()...,
                        displayedComponents: .date
                    )
                }
            }
        }
        .navigationTitle("New Goal")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button("Cancel") { dismiss() }
            }
            ToolbarItem(placement: .primaryAction) {
                Button("Create") {
                    createGoal()
                }
                .disabled(!isValid)
                .fontWeight(.semibold)
            }
        }
        .sheet(isPresented: $showItemPicker) {
            NavigationStack {
                GoalItemPicker(items: items) { itemId in
                    selectedItemId = itemId
                }
            }
        }
    }

    private func createGoal() {
        let kind: GoalKind
        switch selectedKind {
        case .sessionFrequency:
            kind = .sessionFrequency(targetDaysPerWeek: UInt8(targetDaysPerWeek))
        case .practiceTime:
            kind = .practiceTime(targetMinutesPerWeek: UInt32(targetMinutesPerWeek))
        case .itemMastery:
            guard let itemId = selectedItemId else { return }
            kind = .itemMastery(itemId: itemId, targetScore: UInt8(targetScore))
        case .milestone:
            kind = .milestone(description: milestoneDescription)
        }

        let goal = CreateGoal(
            title: title,
            kind: kind,
            deadline: hasDeadline ? deadline : nil
        )

        core.update(.goal(.add(goal)))
        dismiss()
    }
}

// MARK: - Goal Item Picker

private struct GoalItemPicker: View {
    let items: [LibraryItemView]
    let onSelect: (String) -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var searchText = ""

    private var filtered: [LibraryItemView] {
        guard !searchText.isEmpty else { return items }
        return items.filter {
            $0.title.localizedCaseInsensitiveContains(searchText)
            || $0.subtitle.localizedCaseInsensitiveContains(searchText)
        }
    }

    var body: some View {
        List(filtered, id: \.id) { item in
            Button {
                onSelect(item.id)
                dismiss()
            } label: {
                HStack {
                    VStack(alignment: .leading, spacing: 2) {
                        Text(item.title)
                            .font(.subheadline)
                        if !item.subtitle.isEmpty {
                            Text(item.subtitle)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    }
                    Spacer()
                    TypeBadge(itemType: item.itemType)
                }
            }
        }
        .navigationTitle("Select Item")
        .navigationBarTitleDisplayMode(.inline)
        .searchable(text: $searchText, prompt: "Search library")
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button("Cancel") { dismiss() }
            }
        }
    }
}
