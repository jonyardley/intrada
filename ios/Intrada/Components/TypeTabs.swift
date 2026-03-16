import SwiftUI

/// Segmented control for filtering library items by type.
/// Supports two modes:
/// - **Interactive**: Bound to a FilterTab selection with onChange callback
/// - **Display-only**: Shows the current type without allowing changes (used in edit forms)
struct TypeTabs: View {
    @Binding var selection: FilterTab
    var isInteractive: Bool = true
    /// When false, hides the "All" tab — use in add/edit forms where a specific type must be chosen.
    var showAll: Bool = true

    private var visibleTabs: [FilterTab] {
        showAll ? FilterTab.allCases : FilterTab.allCases.filter { $0 != .all }
    }

    var body: some View {
        HStack(spacing: 4) {
            ForEach(visibleTabs) { tab in
                Button {
                    if isInteractive {
                        selection = tab
                    }
                } label: {
                    Text(tab.rawValue)
                        .font(.subheadline)
                        .fontWeight(.medium)
                        .padding(.horizontal, 16)
                        .padding(.vertical, 8)
                        .frame(maxWidth: .infinity)
                        .foregroundStyle(foregroundColor(for: tab))
                        .background(backgroundColor(for: tab))
                        .clipShape(Capsule())
                }
                .disabled(!isInteractive)
                .buttonStyle(.plain)
            }
        }
        .padding(4)
        .background(Color.surfaceSecondary)
        .clipShape(Capsule())
    }

    private func foregroundColor(for tab: FilterTab) -> Color {
        if tab == selection {
            return isInteractive ? .textPrimary : .textFaint
        }
        return isInteractive ? .textMuted : .textFaint
    }

    private func backgroundColor(for tab: FilterTab) -> Color {
        tab == selection ? .accent : .clear
    }
}

/// Display-only type indicator showing Piece or Exercise.
/// Used in edit forms where the type cannot be changed.
struct TypeTabsDisplayOnly: View {
    let kind: ItemKind

    var body: some View {
        HStack(spacing: 4) {
            tabLabel("Piece", isSelected: kind == .piece)
            tabLabel("Exercise", isSelected: kind == .exercise)
        }
        .padding(4)
        .background(Color.surfaceSecondary)
        .clipShape(Capsule())
    }

    private func tabLabel(_ text: String, isSelected: Bool) -> some View {
        Text(text)
            .font(.subheadline)
            .fontWeight(.medium)
            .padding(.horizontal, 16)
            .padding(.vertical, 8)
            .frame(maxWidth: .infinity)
            .foregroundStyle(isSelected ? Color.textMuted : Color.textFaint)
            .background(isSelected ? Color.surfaceHover : Color.clear)
            .clipShape(Capsule())
    }
}

#Preview("Interactive") {
    struct Preview: View {
        @State private var tab: FilterTab = .all
        var body: some View {
            VStack(spacing: 20) {
                TypeTabs(selection: $tab)
                Text("Selected: \(tab.rawValue)")
                    .foregroundStyle(Color.textSecondary)
            }
            .padding()
            .background(Color.backgroundApp)
        }
    }
    return Preview()
        .preferredColorScheme(.dark)
}

#Preview("Display Only") {
    VStack(spacing: 20) {
        TypeTabsDisplayOnly(kind: .piece)
        TypeTabsDisplayOnly(kind: .exercise)
    }
    .padding()
    .background(Color.backgroundApp)
    .preferredColorScheme(.dark)
}
