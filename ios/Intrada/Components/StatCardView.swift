import SwiftUI

/// Metric display card matching the web's `StatCard` component.
///
/// Shows a title (field-label style), large value, and optional subtitle.
///
///     StatCardView(title: "Practices", value: "12")
///     StatCardView(title: "Streak", value: "5 days", subtitle: "Keep it up!")
struct StatCardView: View {

    let title: String
    let value: String
    var subtitle: String? = nil

    var body: some View {
        VStack(spacing: 4) {
            Text(title)
                .fieldLabelStyle()
                .lineLimit(1)
                .minimumScaleFactor(0.7)

            Text(value)
                .font(.title2)
                .fontWeight(.bold)
                .foregroundStyle(Color.textPrimary)
                .lineLimit(1)
                .minimumScaleFactor(0.8)

            if let subtitle {
                Text(subtitle)
                    .font(.caption)
                    .foregroundStyle(Color.textMuted)
                    .lineLimit(1)
            }
        }
        .frame(maxWidth: .infinity)
        .glassCard(padding: Spacing.cardCompact)
    }
}

#Preview("StatCardView") {
    HStack(spacing: 12) {
        StatCardView(title: "Practices", value: "12")
        StatCardView(title: "Streak", value: "5 days", subtitle: "Keep it up!")
        StatCardView(title: "Minutes", value: "87")
    }
    .padding()
    .background(Color.backgroundApp)
}
