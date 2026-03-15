import SwiftUI

/// Serif page heading matching the web's `PageHeading` component.
///
/// Uses Georgia (serif) for the heading text, matching the web's
/// Source Serif 4 / `font-heading` class.
///
///     PageHeading(text: "Library")
///     PageHeading(text: "Library", subtitle: "Your pieces and exercises")
struct PageHeading: View {

    let text: String
    var subtitle: String? = nil

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(text)
                .font(.heading())
                .fontWeight(.bold)
                .foregroundStyle(Color.textPrimary)

            if let subtitle {
                Text(subtitle)
                    .font(.subheadline)
                    .foregroundStyle(Color.textSecondary)
                    .lineSpacing(2)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

#Preview("PageHeading") {
    VStack(spacing: 24) {
        PageHeading(text: "Library")

        PageHeading(
            text: "Practice",
            subtitle: "Play with intention and focus"
        )
    }
    .padding()
    .background(Color.backgroundApp)
}
