import SwiftUI

/// Post-session review with scoring, notes, and save/discard actions.
struct SessionSummaryView: View {
    @Environment(IntradaCore.self) private var core
    @Environment(\.dismiss) private var dismiss

    @State private var notes = ""
    @State private var showRoutineSave = false
    @State private var routineName = ""

    private var summary: SummaryView? { core.viewModel.summary }
    private var sessionStatus: String { core.viewModel.sessionStatus }

    var body: some View {
        Group {
            if let summary {
                ScrollView {
                    VStack(spacing: 20) {
                        // Header
                        VStack(spacing: 8) {
                            Image(systemName: "checkmark.circle.fill")
                                .font(.system(size: 48))
                                .foregroundStyle(.green)

                            Text("Session Complete")
                                .font(.title2)
                                .fontWeight(.bold)

                            Text(summary.totalDurationDisplay)
                                .font(.title3)
                                .foregroundStyle(.secondary)
                        }
                        .padding(.top, 16)

                        // Entries with scoring
                        VStack(alignment: .leading, spacing: 12) {
                            Text("Review & Score")
                                .font(.headline)

                            ForEach(summary.entries, id: \.id) { entry in
                                SummaryEntryRow(entry: entry)
                            }
                        }

                        // Notes
                        VStack(alignment: .leading, spacing: 8) {
                            Text("Session Notes")
                                .font(.headline)

                            TextEditor(text: $notes)
                                .frame(minHeight: 80)
                                .padding(8)
                                .background(.ultraThinMaterial)
                                .clipShape(RoundedRectangle(cornerRadius: 10))
                                .onChange(of: notes) {
                                    core.update(.session(.setSummaryNotes(notes: notes.isEmpty ? nil : notes)))
                                }
                        }

                        // Actions
                        VStack(spacing: 12) {
                            Button {
                                core.update(.session(.saveSession(now: Date())))
                            } label: {
                                Text("Save Session")
                                    .font(.headline)
                                    .foregroundStyle(.white)
                                    .frame(maxWidth: .infinity)
                                    .padding()
                                    .background(.indigo)
                                    .clipShape(RoundedRectangle(cornerRadius: 12))
                            }

                            HStack(spacing: 16) {
                                Button {
                                    showRoutineSave = true
                                } label: {
                                    Label("Save as Routine", systemImage: "list.bullet.rectangle")
                                        .font(.subheadline)
                                }

                                Button(role: .destructive) {
                                    core.update(.session(.discardSession))
                                } label: {
                                    Text("Discard")
                                        .font(.subheadline)
                                }
                            }
                        }
                    }
                    .padding()
                }
                .navigationTitle("Summary")
                .navigationBarTitleDisplayMode(.inline)
                .alert("Save as Routine", isPresented: $showRoutineSave) {
                    TextField("Routine name", text: $routineName)
                    Button("Save") {
                        guard !routineName.isEmpty else { return }
                        core.update(.routine(.saveSummaryAsRoutine(name: routineName)))
                        routineName = ""
                    }
                    Button("Cancel", role: .cancel) { routineName = "" }
                } message: {
                    Text("Save this session's setlist as a reusable routine.")
                }
                .onAppear {
                    notes = summary.notes ?? ""
                }
                .onChange(of: sessionStatus) { _, newValue in
                    if newValue == "idle" {
                        dismiss()
                    }
                }
            } else {
                ContentUnavailableView("No summary available", systemImage: "doc.text")
            }
        }
    }
}

// MARK: - Summary Entry Row

private struct SummaryEntryRow: View {
    let entry: SetlistEntryView

    @Environment(IntradaCore.self) private var core
    @State private var scoreValue: UInt8
    @State private var tempoText: String
    @State private var entryNotes: String

    init(entry: SetlistEntryView) {
        self.entry = entry
        _scoreValue = State(initialValue: entry.score ?? 0)
        _tempoText = State(initialValue: entry.achievedTempo.map { "\($0)" } ?? "")
        _entryNotes = State(initialValue: entry.notes ?? "")
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                TypeBadge(itemType: entry.itemType)
                Text(entry.itemTitle)
                    .font(.subheadline)
                    .fontWeight(.medium)
                Spacer()
                Text(entry.durationDisplay)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            // Score picker
            HStack {
                Text("Score")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Spacer()
                HStack(spacing: 6) {
                    ForEach(UInt8(1)...5, id: \.self) { value in
                        Button {
                            scoreValue = value
                            core.update(.session(.setEntryScore(entryId: entry.id, score: value)))
                        } label: {
                            Circle()
                                .fill(value <= scoreValue ? Color.indigo : Color.secondary.opacity(0.2))
                                .frame(width: 24, height: 24)
                                .overlay {
                                    Text("\(value)")
                                        .font(.caption2)
                                        .fontWeight(.medium)
                                        .foregroundStyle(value <= scoreValue ? .white : .secondary)
                                }
                        }
                        .buttonStyle(.plain)
                    }
                }
            }

            // Tempo input
            HStack {
                Text("Tempo")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Spacer()
                TextField("BPM", text: $tempoText)
                    .keyboardType(.numberPad)
                    .frame(width: 60)
                    .textFieldStyle(.roundedBorder)
                    .font(.caption)
                    .onChange(of: tempoText) {
                        let tempo = UInt16(tempoText)
                        core.update(.session(.setAchievedTempo(entryId: entry.id, tempo: tempo)))
                    }
            }

            // Notes
            TextField("Notes for this item...", text: $entryNotes)
                .font(.caption)
                .textFieldStyle(.roundedBorder)
                .onChange(of: entryNotes) {
                    core.update(.session(.setEntryNotes(entryId: entry.id, notes: entryNotes.isEmpty ? nil : entryNotes)))
                }
        }
        .padding()
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 10))
    }
}
