import SwiftUI

/// A compact row displaying one setlist entry with drag handle and remove button.
///
/// Supports progressive disclosure — tap to expand and show
/// duration/intention/rep target editing fields.
struct SetlistEntryRow: View {
    let entry: SetlistEntryView
    let isExpanded: Bool
    let onTap: () -> Void
    let onRemove: () -> Void
    let onSetDuration: (UInt32?) -> Void
    let onSetIntention: (String?) -> Void
    let onSetRepTarget: (UInt8?) -> Void

    @State private var durationMinutes: String = ""
    @State private var intentionText: String = ""
    @State private var repTargetText: String = ""
    @FocusState private var focusedField: Field?

    private enum Field { case duration, intention, repTarget }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            // Main row (always visible)
            Button(action: onTap) {
                HStack(spacing: 10) {
                    // Drag handle (visual only — actual drag via .onMove)
                    Image(systemName: "line.3.horizontal")
                        .font(.caption)
                        .foregroundStyle(Color.textFaint)
                        .frame(width: 16, height: 16)

                    // Entry info
                    VStack(alignment: .leading, spacing: 2) {
                        Text(entry.itemTitle)
                            .font(.body)
                            .fontWeight(.medium)
                            .foregroundStyle(Color.textPrimary)
                            .lineLimit(1)

                        Text(metadataText)
                            .font(.caption)
                            .foregroundStyle(Color.textMuted)
                    }

                    Spacer()

                    // Type badge
                    TypeBadge(kind: entry.itemType)

                    // Remove button
                    Button {
                        onRemove()
                    } label: {
                        Image(systemName: "xmark")
                            .font(.caption.weight(.medium))
                            .foregroundStyle(Color.dangerText)
                            .frame(width: 16, height: 16)
                    }
                    .buttonStyle(.plain)
                }
                .padding(.vertical, 10)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)

            // Expanded editing fields (progressive disclosure)
            if isExpanded {
                VStack(spacing: 12) {
                    // Duration
                    HStack {
                        Text("Duration (min)")
                            .font(.caption)
                            .foregroundStyle(Color.textMuted)
                        Spacer()
                        TextField("5", text: $durationMinutes)
                            .font(.caption)
                            .foregroundStyle(Color.textPrimary)
                            .keyboardType(.numberPad)
                            .frame(width: 60)
                            .multilineTextAlignment(.trailing)
                            .padding(.horizontal, 8)
                            .padding(.vertical, 6)
                            .background(Color.surfaceInput)
                            .clipShape(RoundedRectangle(cornerRadius: 6))
                            .overlay(
                                RoundedRectangle(cornerRadius: 6)
                                    .stroke(Color.borderInput, lineWidth: 1)
                            )
                            .focused($focusedField, equals: .duration)
                    }

                    // Intention
                    HStack {
                        Text("Focus")
                            .font(.caption)
                            .foregroundStyle(Color.textMuted)
                        Spacer()
                        TextField("e.g. dynamics", text: $intentionText)
                            .font(.caption)
                            .foregroundStyle(Color.textPrimary)
                            .frame(maxWidth: 160)
                            .multilineTextAlignment(.trailing)
                            .padding(.horizontal, 8)
                            .padding(.vertical, 6)
                            .background(Color.surfaceInput)
                            .clipShape(RoundedRectangle(cornerRadius: 6))
                            .overlay(
                                RoundedRectangle(cornerRadius: 6)
                                    .stroke(Color.borderInput, lineWidth: 1)
                            )
                            .focused($focusedField, equals: .intention)
                            .onSubmit { commitIntention() }
                    }

                    // Rep target
                    HStack {
                        Text("Rep target")
                            .font(.caption)
                            .foregroundStyle(Color.textMuted)
                        Spacer()
                        TextField("3", text: $repTargetText)
                            .font(.caption)
                            .foregroundStyle(Color.textPrimary)
                            .keyboardType(.numberPad)
                            .frame(width: 60)
                            .multilineTextAlignment(.trailing)
                            .padding(.horizontal, 8)
                            .padding(.vertical, 6)
                            .background(Color.surfaceInput)
                            .clipShape(RoundedRectangle(cornerRadius: 6))
                            .overlay(
                                RoundedRectangle(cornerRadius: 6)
                                    .stroke(Color.borderInput, lineWidth: 1)
                            )
                            .focused($focusedField, equals: .repTarget)
                    }
                }
                .padding(.leading, 26) // Align with text (past drag handle)
                .padding(.bottom, 10)
                .transition(.opacity.combined(with: .move(edge: .top)))
            }
        }
        .onAppear {
            // Initialise text fields from entry data
            if let secs = entry.plannedDurationSecs {
                durationMinutes = "\(secs / 60)"
            }
            if let intention = entry.intention {
                intentionText = intention
            }
            if let target = entry.repTarget {
                repTargetText = "\(target)"
            }
        }
        .onChange(of: focusedField) { oldField, _ in
            // Commit the value of the field the user just left
            switch oldField {
            case .duration: commitDuration()
            case .intention: commitIntention()
            case .repTarget: commitRepTarget()
            case nil: break
            }
        }
        .onChange(of: isExpanded) {
            // Commit all fields when the row collapses
            if !isExpanded {
                commitDuration()
                commitIntention()
                commitRepTarget()
            }
        }
        .accessibilityElement(children: .combine)
        .accessibilityLabel("\(entry.itemTitle), \(metadataText)")
    }

    // MARK: - Commit Helpers

    private func commitDuration() {
        if let mins = UInt32(durationMinutes), mins > 0 {
            onSetDuration(mins * 60)
        } else {
            onSetDuration(nil)
        }
    }

    private func commitIntention() {
        onSetIntention(intentionText.isEmpty ? nil : intentionText)
    }

    private func commitRepTarget() {
        if let reps = UInt8(repTargetText), reps > 0 {
            onSetRepTarget(reps)
        } else {
            onSetRepTarget(nil)
        }
    }

    private var metadataText: String {
        var parts: [String] = []
        if let display = entry.plannedDurationDisplay {
            parts.append(display)
        }
        if let intention = entry.intention, !intention.isEmpty {
            parts.append(intention)
        }
        if let target = entry.repTarget {
            parts.append("\(target) reps")
        }
        return parts.isEmpty ? "Tap to set details" : parts.joined(separator: " · ")
    }
}

#Preview {
    let sampleEntry = SetlistEntryView(
        id: "e1", itemId: "i1", itemTitle: "Clair de Lune",
        itemType: .piece, position: 0, durationDisplay: "0:00",
        status: .notAttempted, notes: nil, score: nil,
        intention: "Focus on pedalling", repTarget: nil,
        repCount: nil, repTargetReached: nil, repHistory: nil,
        plannedDurationSecs: 300, plannedDurationDisplay: "5 min",
        achievedTempo: nil
    )
    let exerciseEntry = SetlistEntryView(
        id: "e2", itemId: "i2", itemTitle: "Hanon No. 1",
        itemType: .exercise, position: 1, durationDisplay: "0:00",
        status: .notAttempted, notes: nil, score: nil,
        intention: nil, repTarget: 3,
        repCount: nil, repTargetReached: nil, repHistory: nil,
        plannedDurationSecs: 600, plannedDurationDisplay: "10 min",
        achievedTempo: nil
    )

    VStack(spacing: 0) {
        Text("Collapsed").font(.caption2).foregroundStyle(Color.textFaint).padding(.top, 8)
        SetlistEntryRow(
            entry: sampleEntry, isExpanded: false,
            onTap: {}, onRemove: {},
            onSetDuration: { _ in }, onSetIntention: { _ in }, onSetRepTarget: { _ in }
        )
        .padding(.horizontal, 20)

        Divider().background(Color.borderDefault).padding(.leading, 20)

        Text("Expanded").font(.caption2).foregroundStyle(Color.textFaint).padding(.top, 8)
        SetlistEntryRow(
            entry: exerciseEntry, isExpanded: true,
            onTap: {}, onRemove: {},
            onSetDuration: { _ in }, onSetIntention: { _ in }, onSetRepTarget: { _ in }
        )
        .padding(.horizontal, 20)
    }
    .background(Color.backgroundApp)
    .preferredColorScheme(.dark)
}
