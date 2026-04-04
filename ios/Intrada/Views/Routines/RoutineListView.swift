import SwiftUI

/// Routines tab root — list of saved routines with iPad split view.
struct RoutineListView: View {
    @Environment(IntradaCore.self) private var core
    @Environment(\.horizontalSizeClass) private var sizeClass

    @State private var selectedRoutineId: String? = nil
    @State private var deleteRoutineId: String? = nil
    @State private var showDeleteConfirmation: Bool = false

    private var routines: [RoutineView] {
        core.viewModel.routines
    }

    private var selectedRoutine: RoutineView? {
        guard let id = selectedRoutineId else { return nil }
        return routines.first(where: { $0.id == id })
    }

    var body: some View {
        Group {
            if sizeClass == .regular {
                iPadLayout
            } else {
                iPhoneLayout
            }
        }
        .confirmationDialog(
            "Delete this routine?",
            isPresented: $showDeleteConfirmation,
            titleVisibility: .visible
        ) {
            Button("Delete", role: .destructive) {
                if let id = deleteRoutineId {
                    core.update(.routine(.deleteRoutine(id: id)))
                }
                deleteRoutineId = nil
            }
            Button("Cancel", role: .cancel) {
                deleteRoutineId = nil
            }
        } message: {
            Text("This cannot be undone.")
        }
    }

    // MARK: - iPhone

    private var iPhoneLayout: some View {
        NavigationStack {
            Group {
                if routines.isEmpty {
                    emptyState
                } else {
                    routineList
                }
            }
            .background(Color.backgroundApp)
            .navigationTitle("Routines")
            .navigationDestination(for: String.self) { routineId in
                if let routine = routines.first(where: { $0.id == routineId }) {
                    RoutineDetailView(routine: routine)
                }
            }
        }
    }

    // MARK: - iPad

    private var iPadLayout: some View {
        NavigationSplitView {
            Group {
                if routines.isEmpty {
                    emptyState
                } else {
                    routineList
                }
            }
            .background(Color.backgroundApp)
            .navigationTitle("Routines")
        } detail: {
            if let routine = selectedRoutine {
                RoutineDetailView(routine: routine)
            } else {
                EmptyStateView(
                    icon: "hand.tap",
                    title: "Select a routine",
                    message: "Tap a routine to view its details"
                )
            }
        }
    }

    // MARK: - List

    private var routineList: some View {
        List(selection: sizeClass == .regular ? $selectedRoutineId : nil) {
            ForEach(routines, id: \.id) { (routine: RoutineView) in
                Group {
                    if sizeClass == .regular {
                        routineRow(routine)
                            .tag(routine.id)
                    } else {
                        NavigationLink(value: routine.id) {
                            routineRow(routine)
                        }
                    }
                }
                .listRowBackground(Color.surfaceSecondary)
                .swipeActions(edge: .trailing) {
                    Button(role: .destructive) {
                        deleteRoutineId = routine.id
                        showDeleteConfirmation = true
                    } label: {
                        Label("Delete", systemImage: "trash")
                    }
                }
            }
        }
        .listStyle(.plain)
        .scrollContentBackground(.hidden)
    }

    // MARK: - Row

    @ViewBuilder
    private func routineRow(_ routine: RoutineView) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text(routine.name)
                    .font(.system(size: 16, weight: .semibold))
                    .foregroundStyle(Color.textPrimary)
                    .lineLimit(1)

                Spacer()

                Text("\(routine.entryCount) items")
                    .font(.system(size: 13))
                    .foregroundStyle(Color.textMuted)
            }

            // Item names preview
            let itemNames = routine.entries.prefix(3).map(\.itemTitle).joined(separator: " · ")
            if !itemNames.isEmpty {
                Text(itemNames)
                    .font(.system(size: 11))
                    .foregroundStyle(Color.textFaint)
                    .lineLimit(1)
            }
        }
        .padding(.vertical, 4)
    }

    // MARK: - Empty State

    private var emptyState: some View {
        EmptyStateView(
            icon: "list.bullet.rectangle",
            title: "No routines yet",
            message: "Save a practice session as a routine to quickly repeat it"
        )
    }
}

#Preview("RoutineListView") {
    RoutineListView()
        .environment(IntradaCore())
        .preferredColorScheme(.dark)
}
