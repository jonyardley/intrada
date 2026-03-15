import SwiftUI

// MARK: - Design Tokens: Colours
//
// All values converted from oklch (input.css) to sRGB.
// See specs/194-ios-design-system/research.md for conversion table.
//
// Rule: NEVER use raw SwiftUI colours (.white, .indigo, .gray) in views.
// Always use these named tokens.

extension Color {

    // MARK: - Text

    /// Headings, titles, emphasis — oklch(100% 0 0)
    static let textPrimary = Color(red: 1.000, green: 1.000, blue: 1.000)

    /// Body text, descriptions — oklch(86.9% 0.022 252.894)
    static let textSecondary = Color(red: 0.792, green: 0.836, blue: 0.888)

    /// Form labels — oklch(92.9% 0.013 255.508)
    static let textLabel = Color(red: 0.886, green: 0.910, blue: 0.943)

    /// Hints, captions, metadata — oklch(70.4% 0.04 256.788)
    static let textMuted = Color(red: 0.565, green: 0.632, blue: 0.725)

    /// Timestamps, very subtle text — oklch(55.4% 0.046 257.417)
    static let textFaint = Color(red: 0.384, green: 0.455, blue: 0.557)

    // MARK: - Accent (Warm Indigo)

    /// Primary buttons, active tabs — oklch(50.5% 0.24 274)
    static let accent = Color(red: 0.271, green: 0.265, blue: 0.915)

    /// Button hover/pressed — oklch(58% 0.22 274)
    static let accentHover = Color(red: 0.345, green: 0.389, blue: 0.976)

    /// Active nav, links, accent text — oklch(78% 0.11 274)
    static let accentText = Color(red: 0.633, green: 0.699, blue: 0.996)

    /// Focus rings — oklch(66% 0.17 274)
    static let accentFocus = Color(red: 0.454, green: 0.525, blue: 0.975)

    // MARK: - Warm Accent (Gold)

    /// Achievements, streaks — oklch(70% 0.12 78)
    static let warmAccent = Color(red: 0.781, green: 0.581, blue: 0.236)

    /// Gold hover — oklch(76% 0.10 78)
    static let warmAccentHover = Color(red: 0.838, green: 0.655, blue: 0.355)

    /// Gold text on dark — oklch(84% 0.08 80)
    static let warmAccentText = Color(red: 0.903, green: 0.774, blue: 0.560)

    /// Gold surface (12% opacity)
    static let warmAccentSurface = Color(red: 0.781, green: 0.581, blue: 0.236, opacity: 0.12)

    // MARK: - Success (Warm Green/Teal)

    /// Positive actions — oklch(62% 0.16 158)
    static let success = Color(red: 0.000, green: 0.634, blue: 0.374)

    /// Success hover — oklch(68% 0.14 158)
    static let successHover = Color(red: 0.157, green: 0.710, blue: 0.471)

    /// Green text on dark — oklch(79% 0.12 160)
    static let successText = Color(red: 0.431, green: 0.825, blue: 0.627)

    /// Success surface (10% opacity)
    static let successSurface = Color(red: 0.000, green: 0.634, blue: 0.374, opacity: 0.10)

    // MARK: - Warning (Amber)

    /// Heads-up alerts — oklch(74% 0.14 62)
    static let warning = Color(red: 0.913, green: 0.583, blue: 0.256)

    /// Warning text on dark — oklch(83% 0.11 65)
    static let warningText = Color(red: 0.976, green: 0.723, blue: 0.473)

    /// Warning surface (10% opacity)
    static let warningSurface = Color(red: 0.913, green: 0.583, blue: 0.256, opacity: 0.10)

    // MARK: - Danger (Coral)

    /// Danger actions — oklch(63% 0.17 18)
    static let danger = Color(red: 0.865, green: 0.331, blue: 0.381)

    /// Danger hover — oklch(69% 0.15 18)
    static let dangerHover = Color(red: 0.914, green: 0.440, blue: 0.469)

    /// Danger text on dark — oklch(77% 0.13 16)
    static let dangerText = Color(red: 0.987, green: 0.565, blue: 0.595)

    /// Danger surface (10% opacity)
    static let dangerSurface = Color(red: 0.865, green: 0.331, blue: 0.381, opacity: 0.10)

    // MARK: - Info (Blue)

    /// Informational — oklch(62% 0.14 238)
    static let info = Color(red: 0.000, green: 0.565, blue: 0.815)

    /// Info text on dark — oklch(79% 0.11 240)
    static let infoText = Color(red: 0.458, green: 0.768, blue: 0.981)

    /// Info surface (10% opacity)
    static let infoSurface = Color(red: 0.000, green: 0.565, blue: 0.815, opacity: 0.10)

    // MARK: - Surfaces

    /// Glassmorphism cards (white @ 12%)
    static let surfacePrimary = Color.white.opacity(0.12)

    /// Subtle cards, skeletons (white @ 5%)
    static let surfaceSecondary = Color.white.opacity(0.05)

    /// Header/tab bar chrome — oklch(15.6% 0.011 261.692) @ 60%
    static let surfaceChrome = Color(red: 0.038, green: 0.049, blue: 0.067).opacity(0.60)

    /// No-blur fallback — oklch(25.7% 0.09 281.288) @ 80%
    static let surfaceFallback = Color(red: 0.117, green: 0.103, blue: 0.301).opacity(0.80)

    /// Hover states (white @ 22%)
    static let surfaceHover = Color.white.opacity(0.22)

    /// Form inputs (white @ 10%)
    static let surfaceInput = Color.white.opacity(0.10)

    // MARK: - Borders

    /// Separators, list borders (white @ 10%)
    static let borderDefault = Color.white.opacity(0.10)

    /// Card borders (white @ 15%)
    static let borderCard = Color.white.opacity(0.15)

    /// Form input borders (white @ 12%)
    static let borderInput = Color.white.opacity(0.12)

    // MARK: - Badges

    /// Piece badge background (accent @ 18%)
    static let badgePieceBg = Color(red: 0.271, green: 0.265, blue: 0.915).opacity(0.18)

    /// Piece badge text — same as accentText
    static let badgePieceText = Color(red: 0.633, green: 0.699, blue: 0.996)

    /// Exercise badge background (warm accent @ 18%)
    static let badgeExerciseBg = Color(red: 0.781, green: 0.581, blue: 0.236).opacity(0.18)

    /// Exercise badge text — same as warmAccentText
    static let badgeExerciseText = Color(red: 0.903, green: 0.774, blue: 0.560)

    // MARK: - Progress

    /// Progress track (white @ 6%)
    static let progressTrack = Color.white.opacity(0.06)

    /// Progress fill (accent)
    static let progressFill = Color(red: 0.271, green: 0.265, blue: 0.915)

    /// Progress complete (success)
    static let progressComplete = Color(red: 0.000, green: 0.634, blue: 0.374)
}

// MARK: - Preview

#Preview("Colour Tokens") {
    ScrollView {
        VStack(alignment: .leading, spacing: 16) {
            Text("Text Colours")
                .font(.headline)
                .foregroundStyle(Color.textPrimary)

            HStack(spacing: 8) {
                colourSwatch("Primary", Color.textPrimary)
                colourSwatch("Secondary", Color.textSecondary)
                colourSwatch("Label", Color.textLabel)
                colourSwatch("Muted", Color.textMuted)
                colourSwatch("Faint", Color.textFaint)
            }

            Text("Accent Colours")
                .font(.headline)
                .foregroundStyle(Color.textPrimary)

            HStack(spacing: 8) {
                colourSwatch("Accent", Color.accent)
                colourSwatch("Hover", Color.accentHover)
                colourSwatch("Text", Color.accentText)
                colourSwatch("Focus", Color.accentFocus)
            }

            Text("Status Colours")
                .font(.headline)
                .foregroundStyle(Color.textPrimary)

            HStack(spacing: 8) {
                colourSwatch("Success", Color.success)
                colourSwatch("Warning", Color.warning)
                colourSwatch("Danger", Color.danger)
                colourSwatch("Info", Color.info)
            }

            Text("Surfaces")
                .font(.headline)
                .foregroundStyle(Color.textPrimary)

            HStack(spacing: 8) {
                colourSwatch("Primary", Color.surfacePrimary)
                colourSwatch("Secondary", Color.surfaceSecondary)
                colourSwatch("Hover", Color.surfaceHover)
                colourSwatch("Input", Color.surfaceInput)
            }
        }
        .padding()
    }
    .background(Color(red: 0.05, green: 0.05, blue: 0.10))
}

private func colourSwatch(_ label: String, _ colour: Color) -> some View {
    VStack(spacing: 4) {
        RoundedRectangle(cornerRadius: 8)
            .fill(colour)
            .frame(width: 56, height: 40)
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(Color.white.opacity(0.2), lineWidth: 1)
            )
        Text(label)
            .font(.system(size: 9))
            .foregroundStyle(Color.textMuted)
    }
}
