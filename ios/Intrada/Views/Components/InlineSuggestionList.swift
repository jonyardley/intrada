import SwiftUI

/// The inline reveal list shared by `AutocompleteField` and `TagChipInput`:
/// a hairline-topped column of tappable suggestions over a sunken fill. The
/// owner supplies the matches, leading glyph, pick action, and a11y hint.
struct InlineSuggestionList: View {
  let matches: [String]
  let systemImage: String
  let accessibilityHint: (String) -> String
  let onPick: (String) -> Void

  var body: some View {
    VStack(spacing: 0) {
      HairlineDivider()
      ForEach(matches, id: \.self) { suggestion in
        Button {
          onPick(suggestion)
          UISelectionFeedbackGenerator().selectionChanged()
        } label: {
          HStack(spacing: 10) {
            Image(systemName: systemImage)
              .font(IntradaFont.meta)
              .foregroundStyle(IntradaColor.inkFaint)
            Text(suggestion)
              .font(IntradaFont.body)
              .foregroundStyle(IntradaColor.ink)
            Spacer(minLength: 0)
          }
          .padding(.vertical, 10)
          .padding(.horizontal, 16)
          .frame(maxWidth: .infinity, alignment: .leading)
          .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityHint(accessibilityHint(suggestion))

        if suggestion != matches.last {
          HairlineDivider().padding(.leading, 16)
        }
      }
    }
    .background(IntradaColor.surfaceSunken.opacity(0.6))
  }
}
