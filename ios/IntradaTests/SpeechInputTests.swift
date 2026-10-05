import SwiftUI
import Testing

@testable import Intrada

@MainActor
private final class StubRecogniser: SpeechRecogniser {
  var access: SpeechAccess
  var startError: Error?
  var whileAuthorising: (() -> Void)?
  private(set) var stopCount = 0
  private(set) var startCount = 0
  private var onText: (@MainActor @Sendable (String) -> Void)?
  private var onEnd: (@MainActor @Sendable (Bool) -> Void)?

  init(access: SpeechAccess = .allowed, startError: Error? = nil) {
    self.access = access
    self.startError = startError
  }

  func authorise() async -> SpeechAccess {
    whileAuthorising?()
    return access
  }

  func start(
    onText: @escaping @MainActor @Sendable (String) -> Void,
    onEnd: @escaping @MainActor @Sendable (Bool) -> Void
  ) throws {
    if let startError { throw startError }
    startCount += 1
    self.onText = onText
    self.onEnd = onEnd
  }

  func stop() { stopCount += 1 }

  func hear(_ text: String) { onText?(text) }
  func end(failed: Bool) { onEnd?(failed) }
}

@MainActor
private final class Field {
  var text: String
  init(_ text: String) { self.text = text }
}

struct SpeechMergeTests {
  @Test(arguments: [
    ("", "Left hand rushed in bar 12", "Left hand rushed in bar 12"),
    ("Slow practice.", "Got it at 84", "Slow practice. Got it at 84"),
    ("Slow practice. ", "Got it at 84", "Slow practice. Got it at 84"),
    ("Slow practice\n", "Got it at 84", "Slow practice\nGot it at 84"),
    ("bar 12", "left hand rushed", "bar 12 left hand rushed"),
    ("bar 12", "  Left hand rushed  ", "bar 12 Left hand rushed"),
    ("Keep this", "", "Keep this"),
    ("Keep this", "   ", "Keep this"),
    ("", "", ""),
  ])
  func mergesSpokenWordsAfterTheNote(existing: String, spoken: String, expected: String) {
    #expect(SpeechMerge.merge(existing, spoken) == expected)
  }
}

@MainActor
struct SpeechInputModelTests {
  @Test func partialsReplaceEachOtherAfterTheTypedNote() async {
    let stub = StubRecogniser()
    let model = SpeechInputModel(recogniser: stub)
    let field = Field("Tone was thin.")

    await model.start(current: field.text) { field.text = $0 }
    #expect(model.isListening)
    stub.hear("Left hand")
    stub.hear("Left hand rushed in bar 12")

    #expect(field.text == "Tone was thin. Left hand rushed in bar 12")
  }

  @Test func stoppingEndsListeningAndIgnoresLateWords() async {
    let stub = StubRecogniser()
    let model = SpeechInputModel(recogniser: stub)
    let field = Field("")

    await model.start(current: field.text) { field.text = $0 }
    stub.hear("Got it at 84")
    model.stop()
    stub.hear("Got it at 84 then lost it")

    #expect(model.phase == .idle)
    #expect(stub.stopCount == 1)
    #expect(field.text == "Got it at 84")
  }

  @Test(arguments: [
    (SpeechAccess.microphoneDenied, SpeechInputProblem.microphoneDenied),
    (.speechDenied, .speechDenied),
    (.unavailable, .unavailable),
  ])
  func refusedAccessExplainsAndLeavesTheNote(access: SpeechAccess, problem: SpeechInputProblem)
    async
  {
    let stub = StubRecogniser(access: access)
    let model = SpeechInputModel(recogniser: stub)
    let field = Field("Typed by hand")

    await model.start(current: field.text) { field.text = $0 }

    #expect(model.problem == problem)
    #expect(model.phase == .idle)
    #expect(stub.startCount == 0)
    #expect(field.text == "Typed by hand")
  }

  @Test func onlyRefusedAccessOffersSettings() {
    #expect(SpeechInputProblem.microphoneDenied.offersSettings)
    #expect(SpeechInputProblem.speechDenied.offersSettings)
    #expect(!SpeechInputProblem.unavailable.offersSettings)
    #expect(!SpeechInputProblem.nothingHeard.offersSettings)
  }

  @Test func aMicrophoneThatWillNotStartSaysSo() async {
    let stub = StubRecogniser(startError: SpeechRecogniserError.noMicrophoneInput)
    let model = SpeechInputModel(recogniser: stub)

    await model.start(current: "") { _ in }

    #expect(model.problem == .unavailable)
    #expect(model.phase == .idle)
  }

  @Test func aFailureWithNothingHeardSaysSo() async {
    let stub = StubRecogniser()
    let model = SpeechInputModel(recogniser: stub)

    await model.start(current: "") { _ in }
    stub.end(failed: true)

    #expect(model.problem == .nothingHeard)
    #expect(model.phase == .idle)
  }

  @Test func aFailureAfterWordsKeepsThemQuietly() async {
    let stub = StubRecogniser()
    let model = SpeechInputModel(recogniser: stub)
    let field = Field("")

    await model.start(current: field.text) { field.text = $0 }
    stub.hear("Bar 12 at 84")
    stub.end(failed: true)

    #expect(model.problem == nil)
    #expect(model.phase == .idle)
    #expect(field.text == "Bar 12 at 84")
  }

  @Test func aFinalResultStopsListening() async {
    let stub = StubRecogniser()
    let model = SpeechInputModel(recogniser: stub)

    await model.start(current: "") { _ in }
    stub.hear("Done")
    stub.end(failed: false)

    #expect(model.phase == .idle)
    #expect(stub.stopCount == 1)
    #expect(model.problem == nil)
  }

  @Test func toggleStartsThenStops() async {
    let stub = StubRecogniser()
    let model = SpeechInputModel(recogniser: stub)

    await model.toggle(current: "") { _ in }
    #expect(model.isListening)
    await model.toggle(current: "") { _ in }

    #expect(model.phase == .idle)
    #expect(stub.stopCount == 1)
  }

  @Test func thePermissionAlertsDoNotStopAStart() async {
    let stub = StubRecogniser()
    let model = SpeechInputModel(recogniser: stub)
    stub.whileAuthorising = { model.sceneChanged(to: .inactive) }

    await model.start(current: "") { _ in }

    #expect(model.isListening)
    #expect(stub.startCount == 1)
  }

  @Test func goingToTheBackgroundStopsListening() async {
    let stub = StubRecogniser()
    let model = SpeechInputModel(recogniser: stub)

    await model.start(current: "") { _ in }
    model.sceneChanged(to: .background)

    #expect(model.phase == .idle)
    #expect(stub.stopCount == 1)
  }
}
