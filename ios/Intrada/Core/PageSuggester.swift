import FoundationModels
import SharedTypes
import os

private let logger = Logger(subsystem: "com.intrada.native", category: "recognition")

/// Phase C of `specs/piece-from-photo.md`: accuracy, not capability. Every
/// device still gets the core's heuristics, and the core discards any
/// suggestion the page does not literally carry (decision 5).
enum PageSuggester {
  /// The generated bridge types are not `Sendable`, so what crosses back off
  /// the model's task is this, and `SuggestedFields` is built from it.
  struct Suggestion: Sendable {
    var title: String?
    var composer: String?
    var tempoMarking: String?
    var bpm: Int?

    init(
      title: String? = nil, composer: String? = nil, tempoMarking: String? = nil, bpm: Int? = nil
    ) {
      self.title = title
      self.composer = composer
      self.tempoMarking = tempoMarking
      self.bpm = bpm
    }
  }

  /// Foundation Models has no timeout of its own, and the form waits behind a
  /// spinner. Whether this stops the model or only the wait is #1472.
  private static let budget = Duration.seconds(12)

  /// Its own function so a test can reach it: `#available` answers first on a
  /// simulator, and would answer the same whether this held or not.
  static func hasSomethingToRead(_ lines: [String]) -> Bool {
    lines.contains { !$0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }
  }

  static func suggest(readingFrom lines: [String]) async -> Suggestion? {
    guard hasSomethingToRead(lines), !UITestFlags.onDeviceModelDisabled else { return nil }
    guard #available(iOS 26, *) else { return nil }
    return await OnDeviceModel.suggest(from: lines.joined(separator: "\n"), within: budget)
  }
}

extension PageSuggester.Suggestion {
  /// Whether a suggestion is any good is the core's judgement, not the shell's
  /// (spec decision 4), so nothing here inspects one.
  var bridged: SuggestedFields {
    SuggestedFields(
      title: title,
      composer: composer,
      tempoMarking: tempoMarking,
      bpm: bpm.flatMap(UInt16.init(exactly:)),
      chartText: nil)
  }
}

@available(iOS 26, *)
private enum OnDeviceModel {
  @Generable
  struct Fields {
    @Guide(description: "The title of the piece, copied exactly as it appears")
    var title: String?
    // The core strips a credit prefix anyway (`clamped_composer`); asking for
    // it without one just saves the round trip.
    @Guide(
      description:
        "Who wrote it, copied exactly as it appears, without any 'Music by' or 'by' in front of it")
    var composer: String?
    @Guide(
      description:
        "The tempo or style word above the first stave, such as Allegro, Andante or Medium Swing")
    var tempoMarking: String?
    @Guide(description: "The number printed after an equals sign, if there is one")
    var bpm: Int?
  }

  static func suggest(from page: String, within budget: Duration) async -> PageSuggester.Suggestion?
  {
    guard SystemLanguageModel.default.availability == .available else { return nil }

    return await withTaskGroup(of: PageSuggester.Suggestion?.self) { group in
      group.addTask { await respond(to: page) }
      group.addTask {
        try? await Task.sleep(for: budget)
        return nil
      }
      let first = await group.next() ?? nil
      group.cancelAll()
      return first
    }
  }

  private static func respond(to page: String) async -> PageSuggester.Suggestion? {
    let session = LanguageModelSession(instructions: instructions)
    do {
      let fields = try await session.respond(
        to: page,
        generating: Fields.self,
        // Deterministic, because `read_fields` is: rescanning the same page
        // twice should not offer the user two different drafts.
        options: GenerationOptions(sampling: .greedy, maximumResponseTokens: 256)
      ).content
      return PageSuggester.Suggestion(
        title: fields.title,
        composer: fields.composer,
        tempoMarking: fields.tempoMarking,
        bpm: fields.bpm)
    } catch let error as LanguageModelSession.GenerationError {
      // A long page, a page of lyrics, a language the model does not have: all
      // expected, all answered by falling back to the core's heuristics.
      logger.info("page-suggestion declined: \(String(describing: error), privacy: .public)")
      return nil
    } catch {
      report(error, "page-suggestion")
      return nil
    }
  }

  private static let instructions = """
    You are reading the first page of a piece of sheet music. It has been \
    transcribed by optical character recognition, one printed line per line of \
    text, in the order it appears down the page.

    Pick out the title, who wrote it, the tempo word and the metronome mark. \
    Copy each one from the text character for character. Do not translate it, \
    expand it, correct its spelling or tidy it up.

    Leave a field out rather than guessing at it. A field you are unsure of is \
    worth less than no field at all.

    Lyrics are not a title. A publisher, an arranger and a copyright line are \
    not the composer.
    """
}
