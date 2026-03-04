import SwiftUI

/// Active practice session with timer, item display, and scoring controls.
struct ActiveSessionView: View {
    @Environment(IntradaCore.self) private var core
    @Environment(\.dismiss) private var dismiss

    @State private var elapsedSeconds: Int = 0
    @State private var timer: Timer?
    @State private var showEndConfirm = false

    private var session: ActiveSessionData? {
        core.viewModel.activeSession
    }
    private var sessionStatus: String { core.viewModel.sessionStatus }

    var body: some View {
        Group {
            if let session {
                VStack(spacing: 0) {
                    // Progress bar
                    ProgressView(
                        value: Double(session.currentPosition),
                        total: Double(session.totalItems)
                    )
                    .tint(.indigo)

                    ScrollView {
                        VStack(spacing: 24) {
                            // Current item
                            VStack(spacing: 8) {
                                Text("Now Practising")
                                    .font(.caption)
                                    .foregroundStyle(.secondary)

                                Text(session.currentItemTitle)
                                    .font(.title)
                                    .fontWeight(.bold)
                                    .multilineTextAlignment(.center)

                                TypeBadge(itemType: session.currentItemType)

                                Text("\(session.currentPosition + 1) of \(session.totalItems)")
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                            }
                            .padding(.top, 32)

                            // Timer
                            Text(formatTime(elapsedSeconds))
                                .font(.system(size: 56, weight: .thin, design: .rounded))
                                .monospacedDigit()

                            // Rep counter (if applicable)
                            if let repTarget = session.currentRepTarget {
                                RepCounterView(
                                    current: session.currentRepCount ?? 0,
                                    target: repTarget
                                )
                            }

                            // Next item preview
                            if let next = session.nextItemTitle {
                                VStack(spacing: 4) {
                                    Text("Up Next")
                                        .font(.caption2)
                                        .foregroundStyle(.tertiary)
                                    Text(next)
                                        .font(.subheadline)
                                        .foregroundStyle(.secondary)
                                }
                            }

                            Spacer(minLength: 32)
                        }
                    }

                    // Control buttons
                    controlBar(session)
                }
            } else {
                ContentUnavailableView(
                    "No active session",
                    systemImage: "pause.circle",
                    description: Text("Start a new session to begin practising.")
                )
            }
        }
        .navigationTitle("Practice")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button("End") {
                    showEndConfirm = true
                }
                .foregroundStyle(.red)
            }
        }
        .alert("End session early?", isPresented: $showEndConfirm) {
            Button("End Session", role: .destructive) {
                core.update(.session(.endSessionEarly(now: Date())))
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("Your progress will be saved.")
        }
        .onAppear { startTimer() }
        .onDisappear { stopTimer() }
        .onChange(of: sessionStatus) { _, newValue in
            if newValue == "summary" || newValue == "idle" {
                stopTimer()
            }
        }
    }

    // MARK: - Controls

    @ViewBuilder
    private func controlBar(_ session: ActiveSessionData) -> some View {
        HStack(spacing: 20) {
            // Previous
            if session.currentPosition > 0 {
                Button {
                    core.update(.session(.previousItem(now: Date())))
                } label: {
                    Image(systemName: "backward.fill")
                        .font(.title2)
                        .frame(width: 56, height: 56)
                }
                .buttonStyle(.bordered)
                .clipShape(Circle())
            }

            // Skip
            Button {
                core.update(.session(.skipItem(now: Date())))
            } label: {
                Image(systemName: "forward.end.fill")
                    .font(.caption)
                    .frame(width: 44, height: 44)
            }
            .buttonStyle(.bordered)
            .tint(.orange)
            .clipShape(Circle())

            // Next / Finish
            if session.currentPosition + 1 < session.totalItems {
                Button {
                    core.update(.session(.nextItem(now: Date())))
                } label: {
                    Image(systemName: "forward.fill")
                        .font(.title2)
                        .frame(width: 56, height: 56)
                }
                .buttonStyle(.borderedProminent)
                .tint(.indigo)
                .clipShape(Circle())
            } else {
                Button {
                    core.update(.session(.finishSession(now: Date())))
                } label: {
                    Image(systemName: "checkmark")
                        .font(.title2)
                        .frame(width: 56, height: 56)
                }
                .buttonStyle(.borderedProminent)
                .tint(.green)
                .clipShape(Circle())
            }
        }
        .padding()
        .background(.ultraThinMaterial)
    }

    // MARK: - Timer

    private func startTimer() {
        elapsedSeconds = 0
        timer = Timer.scheduledTimer(withTimeInterval: 1, repeats: true) { _ in
            elapsedSeconds += 1
        }
    }

    private func stopTimer() {
        timer?.invalidate()
        timer = nil
    }

    private func formatTime(_ seconds: Int) -> String {
        let h = seconds / 3600
        let m = (seconds % 3600) / 60
        let s = seconds % 60
        if h > 0 {
            return String(format: "%d:%02d:%02d", h, m, s)
        }
        return String(format: "%d:%02d", m, s)
    }
}

// ActiveSessionData is defined in SharedTypes.swift (renamed from ActiveSessionView to avoid collision)

// MARK: - Rep Counter

private struct RepCounterView: View {
    let current: UInt8
    let target: UInt8

    @Environment(IntradaCore.self) private var core

    var body: some View {
        VStack(spacing: 8) {
            Text("Reps")
                .font(.caption)
                .foregroundStyle(.secondary)

            HStack(spacing: 20) {
                Button {
                    core.update(.session(.decrementRep))
                } label: {
                    Image(systemName: "minus.circle.fill")
                        .font(.title2)
                }
                .disabled(current == 0)

                Text("\(current) / \(target)")
                    .font(.title3)
                    .fontWeight(.medium)
                    .monospacedDigit()

                Button {
                    core.update(.session(.incrementRep))
                } label: {
                    Image(systemName: "plus.circle.fill")
                        .font(.title2)
                }
            }
        }
    }
}
