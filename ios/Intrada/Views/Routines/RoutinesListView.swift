import SwiftUI

/// List of saved practice routines with load/edit/delete actions.
struct RoutinesListView: View {
    @Environment(IntradaCore.self) private var core
    @State private var showDeleteConfirm = false
    @State private var routineToDelete: RoutineView?

    private var routines: [RoutineView] {
        core.viewModel.routines
    }

    var body: some View {
        ScrollView {
            if routines.isEmpty {
                EmptyStateView(
                    icon: "list.bullet.rectangle",
                    title: "No routines yet",
                    message: "Save a practice session as a routine to quickly load it again."
                )
            } else {
                LazyVStack(spacing: 12) {
                    ForEach(routines, id: \.id) { routine in
                        NavigationLink(value: routine.id) {
                            RoutineCard(routine: routine)
                        }
                        .buttonStyle(.plain)
                        .contextMenu {
                            Button(role: .destructive) {
                                routineToDelete = routine
                                showDeleteConfirm = true
                            } label: {
                                Label("Delete", systemImage: "trash")
                            }
                        }
                    }
                }
                .padding()

                Text("\(routines.count) routine\(routines.count == 1 ? "" : "s")")
                    .font(.caption)
                    .foregroundStyle(.tertiary)
                    .padding(.bottom)
            }
        }
        .navigationTitle("Routines")
        .navigationDestination(for: String.self) { routineId in
            RoutineEditView(routineId: routineId)
        }
        .alert("Delete Routine?", isPresented: $showDeleteConfirm) {
            Button("Delete", role: .destructive) {
                if let routine = routineToDelete {
                    core.update(.routine(.deleteRoutine(id: routine.id)))
                }
                routineToDelete = nil
            }
            Button("Cancel", role: .cancel) {
                routineToDelete = nil
            }
        } message: {
            if let routine = routineToDelete {
                Text("Are you sure you want to delete \"\(routine.name)\"? This cannot be undone.")
            }
        }
    }
}

// MARK: - Routine Card

private struct RoutineCard: View {
    let routine: RoutineView

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: "list.bullet.rectangle")
                    .foregroundStyle(.indigo)
                Text(routine.name)
                    .font(.headline)
                Spacer()
                Image(systemName: "chevron.right")
                    .font(.caption)
                    .foregroundStyle(.tertiary)
            }

            HStack(spacing: 16) {
                Label("\(routine.entryCount) item\(routine.entryCount == 1 ? "" : "s")", systemImage: "music.note")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            if !routine.entries.isEmpty {
                HStack(spacing: 4) {
                    ForEach(routine.entries.prefix(5), id: \.id) { entry in
                        TypeBadge(itemType: entry.itemType)
                    }
                    if routine.entries.count > 5 {
                        Text("+\(routine.entries.count - 5)")
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                    }
                }
            }
        }
        .padding()
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}
