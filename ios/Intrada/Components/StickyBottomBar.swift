import SwiftUI

/// Persistent bottom bar for the iPhone session builder layout.
///
/// Shows the setlist item count, total estimated time, a hint to
/// edit the setlist, and a "Start Session" button.
struct StickyBottomBar: View {
    let itemCount: Int
    let totalMinutes: Int
    let isDisabled: Bool
    let onTapCount: () -> Void
    let onStartSession: () -> Void

    var body: some View {
        HStack(spacing: 12) {
            // Count + hint (tappable to open sheet)
            Button(action: onTapCount) {
                VStack(alignment: .leading, spacing: 2) {
                    Text(countText)
                        .font(.subheadline.weight(.semibold))
                        .foregroundStyle(Color.textPrimary)

                    if itemCount > 0 {
                        Text("Tap to edit setlist")
                            .font(.caption2)
                            .foregroundStyle(Color.textMuted)
                    }
                }
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .disabled(itemCount == 0)

            Spacer()

            // Start Session button
            Button(action: onStartSession) {
                HStack(spacing: 6) {
                    Text("Start Session")
                        .font(.subheadline.weight(.semibold))
                    Image(systemName: "arrow.right")
                        .font(.caption.weight(.semibold))
                }
                .foregroundStyle(Color.textPrimary)
                .padding(.horizontal, 20)
                .frame(height: 40)
                .background(isDisabled ? Color.accent.opacity(0.4) : Color.accent)
                .clipShape(RoundedRectangle(cornerRadius: 8))
            }
            .disabled(isDisabled)
        }
        .padding(.horizontal, 16)
        .padding(.top, 12)
        .padding(.bottom, 12)
        .background(ignoresSafeAreaEdges: .bottom)
        .background(
            Color.surfaceChrome
                .background(.ultraThinMaterial)
        )
        .overlay(alignment: .top) {
            Rectangle()
                .fill(Color.borderDefault)
                .frame(height: 1)
        }
    }

    private var countText: String {
        if itemCount == 0 {
            return "No items selected"
        } else {
            let itemLabel = itemCount == 1 ? "item" : "items"
            return "\(itemCount) \(itemLabel) · \(totalMinutes) min"
        }
    }
}

#Preview {
    VStack {
        Spacer()

        StickyBottomBar(
            itemCount: 3,
            totalMinutes: 20,
            isDisabled: false,
            onTapCount: {},
            onStartSession: {}
        )

        StickyBottomBar(
            itemCount: 0,
            totalMinutes: 0,
            isDisabled: true,
            onTapCount: {},
            onStartSession: {}
        )
    }
    .background(Color.backgroundApp)
    .preferredColorScheme(.dark)
}
