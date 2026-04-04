import SwiftUI

/// Routine detail — shows name and ordered item list with Edit button.
struct RoutineDetailView: View {
    @Environment(IntradaCore.self) private var core
    let routine: RoutineView

    var body: some View {
        ScrollView {
            VStack(spacing: Spacing.cardCompact) {
                // Header card
                CardView {
                    VStack(alignment: .leading, spacing: 8) {
                        Text(routine.name)
                            .font(.system(size: 20, weight: .bold))
                            .foregroundStyle(Color.textPrimary)

                        Text("\(routine.entryCount) items")
                            .font(.system(size: 13))
                            .foregroundStyle(Color.textMuted)
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
                .padding(.horizontal, Spacing.card)
                .padding(.top, Spacing.card)

                // Entry list card
                CardView(padding: 0) {
                    LazyVStack(spacing: 0) {
                        ForEach(Array(routine.entries.enumerated()), id: \.element.id) { (index: Int, entry: RoutineEntryView) in
                            if index > 0 {
                                Divider().background(Color.borderDefault)
                            }

                            HStack(spacing: Spacing.cardCompact) {
                                Text("\(index + 1)")
                                    .font(.system(size: 13, weight: .medium, design: .monospaced))
                                    .foregroundStyle(Color.textFaint)
                                    .frame(width: 24)

                                Text(entry.itemTitle)
                                    .font(.system(size: 15, weight: .medium))
                                    .foregroundStyle(Color.textPrimary)
                                    .lineLimit(1)

                                Spacer()

                                TypeBadge(kind: entry.itemType)
                            }
                            .padding(.horizontal, Spacing.card)
                            .padding(.vertical, Spacing.cardCompact)
                        }
                    }
                }
                .padding(.horizontal, Spacing.card)
            }
        }
        .background(Color.backgroundApp)
        .navigationTitle("Routine")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                NavigationLink("Edit") {
                    RoutineEditView(routine: routine)
                }
                .font(.system(size: 14, weight: .semibold))
                .foregroundStyle(Color.accentText)
            }
        }
    }
}

#Preview("RoutineDetailView") {
    NavigationStack {
        RoutineDetailView(
            routine: RoutineView(
                id: "r1",
                name: "Morning Warm-up",
                entryCount: 3,
                entries: []
            )
        )
    }
    .environment(IntradaCore())
    .preferredColorScheme(.dark)
}
