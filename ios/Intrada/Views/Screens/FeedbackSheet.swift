import SwiftUI
import UIKit

/// Send feedback during the beta (#598): a note, and from a shake the screen
/// the tester was on, sent to Sentry with the build and recent steps.
struct FeedbackSheet: View {
  @Environment(\.dismiss) private var dismiss
  @Environment(\.dynamicTypeSize) private var typeSize

  let screenshot: Data?
  @State private var note = ""
  @State private var includeScreenshot = true
  @FocusState private var noteFocused: Bool

  init(screenshot: Data? = nil) {
    self.screenshot = screenshot
  }

  private var report: FeedbackReport? {
    FeedbackReport(note: note, screenshot: screenshot, includeScreenshot: includeScreenshot)
  }

  var body: some View {
    NavigationStack {
      ZStack {
        PaperBackground()
        ScrollView {
          VStack(alignment: .leading, spacing: IntradaSpacing.card) {
            FieldCard("Note") {
              TextField("Bug or idea", text: $note, axis: .vertical)
                .lineLimit(4...8)
                .font(IntradaFont.body)
                .foregroundStyle(IntradaColor.ink)
                .focused($noteFocused)
                .accessibilityIdentifier("feedback.note")
            }
            if let image = screenshot.flatMap(UIImage.init(data:)) {
              screenshotCard(image)
            }
          }
          .padding(IntradaSpacing.card)
        }
      }
      .navigationTitle("Feedback")
      .navigationBarTitleDisplayMode(.inline)
      .toolbar {
        ToolbarItem(placement: .cancellationAction) {
          Button("Cancel") { dismiss() }
            .accessibilityIdentifier("feedback.cancel")
        }
        ToolbarItem(placement: .confirmationAction) {
          Button("Send", action: send)
            .disabled(report == nil)
            .accessibilityIdentifier("feedback.send")
        }
      }
    }
    .onAppear { noteFocused = true }
  }

  @ViewBuilder
  private func screenshotCard(_ image: UIImage) -> some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      if typeSize.isAccessibilitySize {
        FieldLabel("Include screenshot")
          .accessibilityHidden(true)
        Toggle("Include screenshot", isOn: $includeScreenshot)
          .labelsHidden()
          .tint(IntradaColor.accent)
      } else {
        Toggle(isOn: $includeScreenshot) { FieldLabel("Include screenshot") }
          .tint(IntradaColor.accent)
      }
      if includeScreenshot {
        Image(uiImage: image)
          .resizable()
          .scaledToFit()
          .frame(maxHeight: 220)
          .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.control))
          .overlay(
            RoundedRectangle(cornerRadius: IntradaRadius.control)
              .stroke(IntradaColor.divider, lineWidth: 1)
          )
          .frame(maxWidth: .infinity)
          .accessibilityLabel("Screenshot of the screen you were on")
      }
    }
    .fieldCardSurface()
  }

  private func send() {
    guard let report else { return }
    FeedbackReport.send(report)
    Haptic.success.play()
    dismiss()
  }
}

#if DEBUG
  #Preview("From Profile") {
    FeedbackSheet()
  }

  #Preview("From a shake") {
    FeedbackSheet(screenshot: UIImage(systemName: "music.note.list")?.pngData())
  }
#endif
