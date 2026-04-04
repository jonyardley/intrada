import SwiftUI

/// Content for the setlist editor — used in bottom sheet (iPhone)
/// and right panel (iPad).
///
/// Shows session intention, setlist entries with drag-to-reorder,
/// total time, and a Start Session button.
struct SetlistSheetContent: View {
    @Environment(IntradaCore.self) private var core
    let onStartSession: () -> Void

    @State private var intentionText: String = ""
    @State private var showRoutinePicker: Bool = false

    private var setlist: BuildingSetlistView? {
        core.viewModel.buildingSetlist
    }

    private var entries: [SetlistEntryView] {
        setlist?.entries ?? []
    }

    private var totalMinutes: Int {
        estimatedTotalMinutes(for: entries)
    }

    var body: some View {
        VStack(spacing: 0) {
            // Header
            HStack {
                Text("Your Setlist")
                    .font(.title2.weight(.bold))
                    .foregroundStyle(Color.textPrimary)

                Spacer()

                if !core.viewModel.routines.isEmpty {
                    Button("Load Routine") {
                        showRoutinePicker = true
                    }
                    .font(.subheadline.weight(.medium))
                    .foregroundStyle(Color.accentText)
                }
            }
            .padding(.horizontal, Spacing.cardComfortable)
            .padding(.top, 16)
            .padding(.bottom, 12)

            // Session intention
            VStack(alignment: .leading, spacing: 6) {
                Text("Session Intention")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(Color.textSecondary)

                TextField("e.g. Focus on dynamics", text: $intentionText)
                    .font(.subheadline)
                    .foregroundStyle(Color.textPrimary)
                    .padding(.horizontal, 12)
                    .frame(height: 40)
                    .background(Color.surfaceInput)
                    .clipShape(RoundedRectangle(cornerRadius: 8))
                    .overlay(
                        RoundedRectangle(cornerRadius: 8)
                            .stroke(Color.borderInput, lineWidth: 1)
                    )
                    .onChange(of: intentionText) {
                        core.update(.session(.setSessionIntention(
                            intention: intentionText.isEmpty ? nil : intentionText
                        )))
                    }
            }
            .padding(.horizontal, Spacing.cardComfortable)
            .padding(.bottom, 16)

            Divider()
                .background(Color.borderDefault)

            // Setlist header
            HStack {
                Text("Setlist (\(entries.count) items · \(totalMinutes) min)")
                    .font(.subheadline.weight(.semibold))
                    .foregroundStyle(Color.textPrimary)
                Spacer()
            }
            .padding(.horizontal, Spacing.cardComfortable)
            .padding(.vertical, 12)

            // Setlist entries with drag-to-reorder
            if entries.isEmpty {
                VStack(spacing: 8) {
                    Text("No items added yet")
                        .font(.subheadline)
                        .foregroundStyle(Color.textMuted)
                    Text("Tap items in the library to add them")
                        .font(.caption)
                        .foregroundStyle(Color.textFaint)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                List {
                    ForEach(entries, id: \.id) { (entry: SetlistEntryView) in
                        SetlistEntryRow(
                            entry: entry,
                            onRemove: {
                                core.update(.session(.removeFromSetlist(entryId: entry.id)))
                            },
                            onSetDuration: { secs in
                                core.update(.session(.setEntryDuration(
                                    entryId: entry.id, durationSecs: secs
                                )))
                            },
                            onSetIntention: { intention in
                                core.update(.session(.setEntryIntention(
                                    entryId: entry.id, intention: intention
                                )))
                            },
                            onSetRepTarget: { target in
                                core.update(.session(.setRepTarget(
                                    entryId: entry.id, target: target
                                )))
                            }
                        )
                        .listRowBackground(Color.clear)
                        .listRowSeparator(.hidden)
                        .listRowInsets(EdgeInsets(top: 0, leading: 20, bottom: 0, trailing: 20))
                    }
                    .onMove { from, to in
                        // SwiftUI gives us source IndexSet and destination Int
                        guard let fromIndex = from.first else { return }
                        let entry = entries[fromIndex]
                        // Calculate the new position accounting for SwiftUI's move semantics
                        let newPosition = fromIndex < to ? to - 1 : to
                        core.update(.session(.reorderSetlist(
                            entryId: entry.id,
                            newPosition: UInt64(newPosition)
                        )))
                    }
                }
                .listStyle(.plain)
                .scrollContentBackground(.hidden)
                .environment(\.editMode, .constant(.active)) // Enable drag handles
            }

            // Total + Start button
            VStack(spacing: 12) {
                HStack {
                    Spacer()
                    Text("Total: \(totalMinutes) min")
                        .font(.caption.weight(.medium))
                        .foregroundStyle(Color.textMuted)
                }

                Button(action: onStartSession) {
                    HStack(spacing: 8) {
                        Text("Start Session")
                            .font(.body.weight(.semibold))
                        Image(systemName: "arrow.right")
                            .font(.subheadline.weight(.semibold))
                    }
                    .foregroundStyle(Color.textPrimary)
                    .frame(maxWidth: .infinity)
                    .frame(height: 48)
                    .background(entries.isEmpty ? Color.accent.opacity(0.4) : Color.accent)
                    .clipShape(RoundedRectangle(cornerRadius: 8))
                }
                .disabled(entries.isEmpty)

                if !entries.isEmpty {
                    RoutineSaveForm { name in
                        core.update(.routine(.saveBuildingAsRoutine(name: name)))
                    }
                }
            }
            .padding(.horizontal, Spacing.cardComfortable)
            .padding(.vertical, 12)
            .padding(.bottom, 20)
        }
        .onAppear {
            intentionText = setlist?.sessionIntention ?? ""
        }
        .sheet(isPresented: $showRoutinePicker) {
            routinePickerSheet
        }
    }

    // MARK: - Routine Picker

    private var routinePickerSheet: some View {
        NavigationStack {
            List {
                ForEach(core.viewModel.routines, id: \.id) { (routine: RoutineView) in
                    Button {
                        core.update(.routine(.loadRoutineIntoSetlist(routineId: routine.id)))
                        showRoutinePicker = false
                    } label: {
                        VStack(alignment: .leading, spacing: 4) {
                            Text(routine.name)
                                .font(.system(size: 15, weight: .semibold))
                                .foregroundStyle(Color.textPrimary)
                            Text("\(routine.entryCount) items")
                                .font(.system(size: 12))
                                .foregroundStyle(Color.textMuted)
                        }
                    }
                    .listRowBackground(Color.surfaceSecondary)
                }
            }
            .listStyle(.plain)
            .scrollContentBackground(.hidden)
            .background(Color.backgroundApp)
            .navigationTitle("Load Routine")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Cancel") {
                        showRoutinePicker = false
                    }
                    .foregroundStyle(Color.accentText)
                }
            }
        }
    }
}

#Preview {
    SetlistSheetContent(
        onStartSession: {}
    )
    .environment(IntradaCore())
    .background(Color.backgroundApp)
    .preferredColorScheme(.dark)
}
