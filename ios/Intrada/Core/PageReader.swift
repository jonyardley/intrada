import SharedTypes
import UIKit
import Vision

/// Runs Vision text recognition over a stored page and hands the core lines
/// with geometry. Owns no interpretation: what a line *means* is
/// `recognition::read_fields` in the core (spec decision 4).
enum PageReader {
  /// What crosses back from the recognition thread. The generated bridge types
  /// are not `Sendable`, so they are built on the main actor from this.
  private struct TextLine: Sendable {
    let text: String
    let x: Float
    let y: Float
    let width: Float
    let height: Float
    let confidence: Float
  }

  @MainActor
  static func read(photoId: String) async -> RecognitionOutput {
    guard let source = try? PhotoFileStore.url(for: photoId) else { return .failed }

    // Decoding a 2048px JPEG and running Vision over it both block for long
    // enough to drop frames, so the whole job runs off the main actor and only
    // the plain values come back.
    guard
      let lines = await Task.detached(priority: .userInitiated, operation: { read(from: source) })
        .value
    else {
      return .failed
    }

    return .page(
      PageReading(
        lines: lines.map {
          RecognisedLine(
            text: $0.text, x: $0.x, y: $0.y, width: $0.width, height: $0.height,
            confidence: $0.confidence)
        },
        // Phase C fills this from Foundation Models; the core produces a usable
        // draft without it on every device we target (spec decision 6).
        suggested: nil
      )
    )
  }

  private nonisolated static func read(from source: URL) -> [TextLine]? {
    guard let image = UIImage(contentsOfFile: source.path), let cgImage = image.cgImage else {
      return nil
    }
    return recognise(cgImage)
  }

  private nonisolated static func recognise(_ cgImage: CGImage) -> [TextLine]? {
    let request = VNRecognizeTextRequest()
    request.recognitionLevel = .accurate
    // A title is as often Italian or German as English.
    request.automaticallyDetectsLanguage = true
    // A stave is not a paragraph: correcting against a lexicon turns chord
    // symbols and tempo markings into ordinary words.
    request.usesLanguageCorrection = false

    do {
      try VNImageRequestHandler(cgImage: cgImage, options: [:]).perform([request])
    } catch {
      report(error, "page-recognition")
      return nil
    }

    return (request.results ?? []).compactMap(line(from:))
  }

  /// Vision normalises to origin bottom-left; the core's contract is top-left.
  private nonisolated static func line(from observation: VNRecognizedTextObservation) -> TextLine? {
    guard let candidate = observation.topCandidates(1).first else { return nil }
    let box = observation.boundingBox
    return TextLine(
      text: candidate.string,
      x: Float(box.origin.x),
      y: Float(1 - (box.origin.y + box.size.height)),
      width: Float(box.size.width),
      height: Float(box.size.height),
      confidence: candidate.confidence
    )
  }
}
