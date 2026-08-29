import FoundationModels
import SharedTypes

/// Asks the on-device model to pick the fields out of what Vision read, on the
/// devices that have one. Phase C of `specs/piece-from-photo.md`: accuracy, not
/// capability. Every device still gets the core's geometry heuristics, and
/// nothing here can invent a field, because the core discards any suggestion
/// the page does not literally carry (spec decision 5).
enum PageSuggester {
  /// Deliberately not the `SuggestedFields` the core takes: that type is only
  /// buildable on the actor the bridge runs on, and this crosses off it.
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

  /// Past this the user has been watching a spinner over an empty form for
  /// long enough that the heuristics, which are already good, are the better
  /// answer. Foundation Models has no timeout of its own.
  private static let budget = Duration.seconds(12)

  /// Checked before availability, so the rule holds on the devices that do have
  /// a model: a blank page gives it nothing to choose from, and anything it
  /// returned could then only be invented.
  static func hasSomethingToRead(_ lines: [String]) -> Bool {
    lines.contains { !$0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }
  }

  static func suggest(readingFrom lines: [String]) async -> Suggestion? {
    guard hasSomethingToRead(lines) else { return nil }
    guard #available(iOS 26, *) else { return nil }
    return await OnDeviceModel.suggest(from: lines.joined(separator: "\n"), within: budget)
  }
}

extension PageSuggester.Suggestion {
  /// Crosses to the core untouched, apart from the bridge's narrower bpm type.
  /// Whether a suggestion is any good is the core's judgement, not the shell's
  /// (spec decision 4).
  var bridged: SuggestedFields {
    SuggestedFields(
      title: title,
      composer: composer,
      tempoMarking: tempoMarking,
      bpm: bpm.flatMap(UInt16.init(exactly:)),
      // Phase D. The model is not asked for a chart.
      chartText: nil)
  }
}

@available(iOS 26, *)
private enum OnDeviceModel {
  @Generable
  struct Fields {
    @Guide(description: "The title of the piece, copied exactly as it appears")
    var title: String?
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
        options: GenerationOptions(sampling: .greedy)
      ).content
      return PageSuggester.Suggestion(
        title: fields.title,
        composer: fields.composer,
        tempoMarking: fields.tempoMarking,
        bpm: fields.bpm)
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
