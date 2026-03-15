import SwiftUI

/// Back-navigation link matching the web's `BackLink` component.
///
///     BackLink(label: "Library") { navigateBack() }
struct BackLink: View {

    let label: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 4) {
                Image(systemName: "chevron.left")
                    .font(.system(size: 14, weight: .medium))
                Text(label)
                    .font(.system(size: 14, weight: .medium))
            }
            .foregroundStyle(Color.textMuted)
        }
    }
}

#Preview("BackLink") {
    VStack(alignment: .leading, spacing: 16) {
        BackLink(label: "Library") { }
        BackLink(label: "Sessions") { }
    }
    .padding()
    .background(Color.backgroundApp)
}
