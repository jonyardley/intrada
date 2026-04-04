import SwiftUI

/// A setlist entry row with always-visible inline chip controls.
///
/// Duration, focus, and rep target are shown as tappable chips below
/// the title — no hidden expand/collapse needed. Matches the web app's
/// progressive disclosure pattern.
struct SetlistEntryRow: View {
    let entry: SetlistEntryView
    let onRemove: () -> Void
    let onSetDuration: (UInt32?) -> Void
    let onSetIntention: (String?) -> Void
    let onSetRepTarget: (UInt8?) -> Void

    @State private var isEditingFocus: Bool = false
    @State private var intentionText: String = ""
    @State private var showDurationPicker: Bool = false
    @State private var showRepPicker: Bool = false
    @FocusState private var focusFieldActive: Bool

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            // Row 1: Title + badge + remove
            HStack(spacing: 10) {
                VStack(alignment: .leading, spacing: 2) {
                    Text(entry.itemTitle)
                        .font(.body)
                        .fontWeight(.medium)
                        .foregroundStyle(Color.textPrimary)
                        .lineLimit(1)
                }

                Spacer()

                TypeBadge(kind: entry.itemType)

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

            // Row 2: Inline chip controls
            FlowLayout(spacing: 6) {
                durationChip
                focusChip
                repChip
            }

            // Row 3: Focus text field (when editing)
            if isEditingFocus {
                HStack(spacing: 8) {
                    TextField("e.g. even tempo, dynamics", text: $intentionText)
                        .font(.caption)
                        .foregroundStyle(Color.textPrimary)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 6)
                        .background(Color.surfaceInput)
                        .clipShape(RoundedRectangle(cornerRadius: 6))
                        .overlay(
                            RoundedRectangle(cornerRadius: 6)
                                .stroke(Color.borderInput, lineWidth: 1)
                        )
                        .focused($focusFieldActive)
                        .onSubmit { commitFocus() }

                    Button {
                        commitFocus()
                    } label: {
                        Text("Done")
                            .font(.caption.weight(.medium))
                            .foregroundStyle(Color.accentText)
                    }
                    .buttonStyle(.plain)
                }
                .transition(.opacity.combined(with: .move(edge: .top)))
            }

            // Duration picker (inline, below chips)
            if showDurationPicker {
                durationPickerRow
                    .transition(.opacity.combined(with: .move(edge: .top)))
            }

            // Rep picker (inline, below chips)
            if showRepPicker {
                repPickerRow
                    .transition(.opacity.combined(with: .move(edge: .top)))
            }
        }
        .padding(.vertical, 8)
        .onAppear {
            if let intention = entry.intention {
                intentionText = intention
            }
            isEditingFocus = false
        }
        .accessibilityElement(children: .combine)
        .accessibilityLabel("\(entry.itemTitle), \(accessibilityDescription)")
    }

    // MARK: - Duration Chip

    @ViewBuilder
    private var durationChip: some View {
        if let secs = entry.plannedDurationSecs {
            // Set: show value chip with remove
            chipButton(
                label: "\(secs / 60) min",
                style: .filled,
                onTap: {
                    withAnimation(.easeInOut(duration: 0.2)) {
                        showDurationPicker.toggle()
                        showRepPicker = false
                    }
                },
                onRemove: { onSetDuration(nil) }
            )
        } else {
            // Not set: show add button
            chipButton(
                label: "+ Duration",
                style: .outline,
                onTap: {
                    onSetDuration(5 * 60) // Default 5 min
                },
                onRemove: nil
            )
        }
    }

    // MARK: - Focus Chip

    @ViewBuilder
    private var focusChip: some View {
        if let intention = entry.intention, !intention.isEmpty {
            // Set: show value chip with remove
            chipButton(
                label: intention,
                style: .filled,
                onTap: {
                    isEditingFocus = true
                    focusFieldActive = true
                    showDurationPicker = false
                    showRepPicker = false
                },
                onRemove: {
                    intentionText = ""
                    isEditingFocus = false
                    onSetIntention(nil)
                }
            )
        } else {
            // Not set: show add button
            chipButton(
                label: "+ Focus",
                style: .outline,
                onTap: {
                    withAnimation(.easeInOut(duration: 0.2)) {
                        isEditingFocus = true
                        focusFieldActive = true
                        showDurationPicker = false
                        showRepPicker = false
                    }
                },
                onRemove: nil
            )
        }
    }

    // MARK: - Rep Chip

    @ViewBuilder
    private var repChip: some View {
        if let target = entry.repTarget {
            // Set: show value chip with remove
            chipButton(
                label: "\(target) reps",
                style: .filled,
                onTap: {
                    withAnimation(.easeInOut(duration: 0.2)) {
                        showRepPicker.toggle()
                        showDurationPicker = false
                    }
                },
                onRemove: { onSetRepTarget(nil) }
            )
        } else {
            // Not set: show add button
            chipButton(
                label: "+ Reps",
                style: .outline,
                onTap: {
                    onSetRepTarget(3) // Default 3 reps
                },
                onRemove: nil
            )
        }
    }

    // MARK: - Inline Pickers

    private var durationPickerRow: some View {
        HStack(spacing: 8) {
            Text("Duration")
                .font(.caption)
                .foregroundStyle(Color.textMuted)

            Picker("Minutes", selection: Binding(
                get: {
                    Int(entry.plannedDurationSecs ?? 300) / 60
                },
                set: { newMins in
                    onSetDuration(UInt32(newMins) * 60)
                }
            )) {
                ForEach(1...60, id: \.self) { mins in
                    Text("\(mins) min").tag(mins)
                }
            }
            .pickerStyle(.menu)
            .tint(Color.accentText)
        }
    }

    private var repPickerRow: some View {
        HStack(spacing: 8) {
            Text("Rep target")
                .font(.caption)
                .foregroundStyle(Color.textMuted)

            Picker("Reps", selection: Binding(
                get: {
                    Int(entry.repTarget ?? 3)
                },
                set: { newTarget in
                    onSetRepTarget(UInt8(newTarget))
                }
            )) {
                ForEach(1...15, id: \.self) { reps in
                    Text("\(reps)").tag(reps)
                }
            }
            .pickerStyle(.menu)
            .tint(Color.accentText)
        }
    }

    // MARK: - Chip Button

    private enum ChipStyle { case filled, outline }

    @ViewBuilder
    private func chipButton(
        label: String,
        style: ChipStyle,
        onTap: @escaping () -> Void,
        onRemove: (() -> Void)?
    ) -> some View {
        HStack(spacing: 4) {
            Button(action: onTap) {
                Text(label)
                    .font(.system(size: 12, weight: .medium))
                    .foregroundStyle(style == .outline ? Color.accentText : Color.textPrimary)
                    .lineLimit(1)
            }
            .buttonStyle(.plain)

            if let onRemove {
                Button(action: onRemove) {
                    Image(systemName: "xmark")
                        .font(.system(size: 8, weight: .bold))
                        .foregroundStyle(Color.textFaint)
                }
                .buttonStyle(.plain)
            }
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 5)
        .background(style == .filled ? Color.surfaceSecondary : Color.clear)
        .clipShape(RoundedRectangle(cornerRadius: DesignRadius.pill))
        .overlay(
            RoundedRectangle(cornerRadius: DesignRadius.pill)
                .stroke(
                    style == .outline ? Color.accentText.opacity(0.4) : Color.borderDefault,
                    lineWidth: 1
                )
        )
    }

    // MARK: - Helpers

    private func commitFocus() {
        onSetIntention(intentionText.isEmpty ? nil : intentionText)
        withAnimation(.easeInOut(duration: 0.2)) {
            isEditingFocus = false
        }
    }

    private var accessibilityDescription: String {
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
        return parts.isEmpty ? "No details set" : parts.joined(separator: ", ")
    }
}

#Preview {
    let noDetails = SetlistEntryView(
        id: "e0", itemId: "i0", itemTitle: "Chromatic Scales",
        itemType: .exercise, position: 0, durationDisplay: "0:00",
        status: .notAttempted, notes: nil, score: nil,
        intention: nil, repTarget: nil,
        repCount: nil, repTargetReached: nil, repHistory: nil,
        plannedDurationSecs: nil, plannedDurationDisplay: nil,
        achievedTempo: nil
    )
    let withDetails = SetlistEntryView(
        id: "e1", itemId: "i1", itemTitle: "Clair de Lune",
        itemType: .piece, position: 1, durationDisplay: "0:00",
        status: .notAttempted, notes: nil, score: nil,
        intention: "Focus on pedalling", repTarget: nil,
        repCount: nil, repTargetReached: nil, repHistory: nil,
        plannedDurationSecs: 300, plannedDurationDisplay: "5 min",
        achievedTempo: nil
    )
    let withReps = SetlistEntryView(
        id: "e2", itemId: "i2", itemTitle: "Hanon No. 1",
        itemType: .exercise, position: 2, durationDisplay: "0:00",
        status: .notAttempted, notes: nil, score: nil,
        intention: nil, repTarget: 3,
        repCount: nil, repTargetReached: nil, repHistory: nil,
        plannedDurationSecs: 600, plannedDurationDisplay: "10 min",
        achievedTempo: nil
    )

    List {
        SetlistEntryRow(
            entry: noDetails,
            onRemove: {},
            onSetDuration: { _ in }, onSetIntention: { _ in }, onSetRepTarget: { _ in }
        )
        SetlistEntryRow(
            entry: withDetails,
            onRemove: {},
            onSetDuration: { _ in }, onSetIntention: { _ in }, onSetRepTarget: { _ in }
        )
        SetlistEntryRow(
            entry: withReps,
            onRemove: {},
            onSetDuration: { _ in }, onSetIntention: { _ in }, onSetRepTarget: { _ in }
        )
    }
    .listStyle(.plain)
    .scrollContentBackground(.hidden)
    .background(Color.backgroundApp)
    .preferredColorScheme(.dark)
}
