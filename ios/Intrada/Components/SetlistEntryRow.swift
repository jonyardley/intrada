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
                            .onChange(of: durationMinutes) {
                                if let mins = UInt32(durationMinutes), mins > 0 {
                                    onSetDuration(mins * 60)
                                } else if durationMinutes.isEmpty {
                                    onSetDuration(nil)
                                }
                            }
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
                            .onChange(of: intentionText) {
                                onSetIntention(intentionText.isEmpty ? nil : intentionText)
                            }
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
                            .onChange(of: repTargetText) {
                                if let reps = UInt8(repTargetText), reps > 0 {
                                    onSetRepTarget(reps)
                                } else if repTargetText.isEmpty {
                                    onSetRepTarget(nil)
                                }
                            }
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
        .accessibilityElement(children: .combine)
        .accessibilityLabel("\(entry.itemTitle), \(metadataText)")
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
    VStack(spacing: 1) {
        Text("SetlistEntryRow Preview")
            .font(.caption)
            .foregroundStyle(Color.textMuted)
            .padding()
    }
    .background(Color.backgroundApp)
    .preferredColorScheme(.dark)
}
