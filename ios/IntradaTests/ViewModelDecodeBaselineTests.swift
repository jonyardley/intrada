import Foundation
import SharedTypes
import Testing

@testable import Intrada

/// #1801 at a realistic library size, through the real bridge. The payload check runs in the
/// fast tier; the timing baseline only when asked:
/// `TEST_RUNNER_INTRADA_BASELINE=1 just ios-test`, numbers in the result's attachment.
@MainActor
struct ViewModelDecodeBaselineTests {
  private static let pieces = 100
  private static let exercises = 100
  private static let pastSessions = 100
  private static let entriesPerSession = 6
  private static let base = Date(timeIntervalSince1970: 1_767_225_600)

  private static func stamp(day: Int, minute: Int) -> String {
    SessionClock.nowRFC3339(base.addingTimeInterval(Double(day) * 86_400 + Double(minute) * 60))
  }

  private static func create(_ title: String, _ kind: ItemKind, variants: [String] = [])
    -> Event
  {
    .item(
      .add(
        CreateItem(
          title: title, kind: kind, composer: kind == .piece ? "Composer \(title.count % 20)" : nil,
          key: nil, modality: nil, tempo: nil,
          notes: "Left hand evenness in the middle section, then hands together slowly.",
          tags: ["grade 8", "exam", "repertoire"], photoId: nil, variantLabels: variants)))
  }

  private static let keys = ["C major", "G major", "D major", "A major", "E major", "B major"]

  private func seededBridge() throws -> (RowsBridge, [String]) {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    for index in 0..<Self.exercises {
      _ = try bridge.update(
        Self.create("Exercise \(index)", .exercise, variants: index % 2 == 0 ? Self.keys : []))
    }
    for index in 0..<Self.pieces {
      _ = try bridge.update(Self.create("Piece \(index)", .piece))
    }
    let view = try bridge.rendered()
    let exerciseIds = view.items.filter { $0.itemType == .exercise }.map(\.id)
    let pieceIds = view.items.filter { $0.itemType == .piece }.map(\.id)
    #expect(exerciseIds.count == Self.exercises)
    #expect(pieceIds.count == Self.pieces)
    for (index, pieceId) in pieceIds.enumerated() {
      for offset in 0..<3 {
        let exerciseId = exerciseIds[(index * 3 + offset) % exerciseIds.count]
        _ = try bridge.update(.item(.linkExercise(pieceId: pieceId, exerciseId: exerciseId)))
      }
    }

    for day in 0..<Self.pastSessions {
      _ = try bridge.update(.session(.startBuilding))
      for offset in 0..<Self.entriesPerSession {
        let itemId = exerciseIds[(day * Self.entriesPerSession + offset) % exerciseIds.count]
        _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
      }
      _ = try bridge.update(.session(.startSession(now: Self.stamp(day: day, minute: 0))))
      for step in 1...Self.entriesPerSession {
        let now = Self.stamp(day: day, minute: step * 5)
        _ = try bridge.update(
          .session(.nextItem(now: now, nextItemStartedAt: now, reading: .silent)))
      }
      let summary = try #require(try bridge.view().summary)
      for entry in summary.entries {
        guard let play = entry.plays.last else { continue }
        _ = try bridge.update(
          .session(.updateEntryScore(entryId: entry.id, playId: play.id, score: 4)))
      }
      let requests = try bridge.update(
        .session(.saveSession(now: Self.stamp(day: day, minute: 40))))
      let write = try #require(
        requests.first {
          if case .persistence(.saveSession) = $0.effect { return true } else { return false }
        })
      _ = try bridge.resolve(write.id, persistenceOutput: .ack)
    }
    let seeded = try bridge.rendered()
    #expect(seeded.error == nil)
    #expect(seeded.sessions.count == Self.pastSessions)
    return (bridge, exerciseIds)
  }

  private static func ms(_ duration: Duration) -> Double {
    Double(duration.components.seconds) * 1_000
      + Double(duration.components.attoseconds) / 1e15
  }

  private static func median(_ samples: [Duration]) -> Double {
    ms(samples.sorted()[samples.count / 2])
  }

  @Test("the screen state sent on Start and on each Next stays under 20 KB")
  func payloadPerTap() throws {
    let (bridge, exerciseIds) = try seededBridge()
    let day = Self.pastSessions
    _ = try bridge.update(.session(.startBuilding))
    for offset in 0..<Self.entriesPerSession {
      _ = try bridge.update(.session(.addToSetlist(itemId: exerciseIds[offset])))
    }
    var taps = [Event.session(.startSession(now: Self.stamp(day: day, minute: 0)))]
    for step in 1..<Self.entriesPerSession {
      let now = Self.stamp(day: day, minute: step * 5)
      taps.append(.session(.nextItem(now: now, nextItemStartedAt: now, reading: .silent)))
    }
    for tap in taps {
      _ = try bridge.update(tap)
      #expect(bridge.lastSections.isEmpty, "\(tap) sent \(bridge.lastSections.count) sections")
      let bytes = try bridge.view().bincodeSerialize().count
      #expect(bytes < 20_000, "\(bytes) bytes after \(tap)")
    }
    #expect(try bridge.view().activeSession != nil)
  }

  @Test(
    "a realistic ViewModel decodes, and the per-event cost is recorded as a baseline",
    .enabled(if: ProcessInfo.processInfo.environment["INTRADA_BASELINE"] != nil))
  func decodeAndRoundTripBaseline() throws {
    let (bridge, exerciseIds) = try seededBridge()
    let clock = ContinuousClock()

    let bytes = try bridge.view().bincodeSerialize()
    var decodes: [Duration] = []
    for _ in 0..<15 {
      decodes.append(clock.measure { _ = try? ViewModel.bincodeDeserialize(input: bytes) })
    }
    #expect(try ViewModel.bincodeDeserialize(input: bytes) == bridge.view())

    var starts: [Duration] = []
    var nexts: [Duration] = []
    for round in 0..<5 {
      let day = Self.pastSessions + round
      _ = try bridge.update(.session(.startBuilding))
      for offset in 0..<Self.entriesPerSession {
        _ = try bridge.update(.session(.addToSetlist(itemId: exerciseIds[offset + round])))
      }
      let start = Event.session(.startSession(now: Self.stamp(day: day, minute: 0)))
      starts.append(
        try clock.measure {
          _ = try bridge.update(start)
          _ = try bridge.view()
        })
      for step in 1..<Self.entriesPerSession {
        let now = Self.stamp(day: day, minute: step * 5)
        let next = Event.session(.nextItem(now: now, nextItemStartedAt: now, reading: .silent))
        nexts.append(
          try clock.measure {
            _ = try bridge.update(next)
            _ = try bridge.view()
          })
      }
      _ = try bridge.update(
        .session(.endSessionEarly(now: Self.stamp(day: day, minute: 45), reading: .silent)))
      _ = try bridge.update(.session(.discardSession))
    }
    #expect(try bridge.view().activeSession == nil)

    let suite = "ViewModelDecodeBaselineTests-\(UUID().uuidString)"
    let defaults = try #require(UserDefaults(suiteName: suite))
    defer { defaults.removePersistentDomain(forName: suite) }
    let store = Store(bridge: bridge, sortDefaults: defaults)
    var storeNexts: [Duration] = []
    let day = Self.pastSessions + 5
    store.send(.session(.startBuilding))
    for offset in 0..<Self.entriesPerSession {
      store.send(.session(.addToSetlist(itemId: exerciseIds[offset])))
    }
    store.send(.session(.startSession(now: Self.stamp(day: day, minute: 0))))
    for step in 1..<Self.entriesPerSession {
      let now = Self.stamp(day: day, minute: step * 5)
      storeNexts.append(
        clock.measure {
          store.send(.session(.nextItem(now: now, nextItemStartedAt: now, reading: .silent)))
        })
    }
    #expect(store.viewModel?.activeSession != nil)

    let decode = Self.median(decodes)
    let report = """
      library items \(Self.pieces + Self.exercises), past sessions \(Self.pastSessions)
      ViewModel bytes \(bytes.count)
      decode median ms \(String(format: "%.2f", decode))
      StartSession bridge round trip median ms \(String(format: "%.2f", Self.median(starts)))
      NextItem bridge round trip median ms \(String(format: "%.2f", Self.median(nexts)))
      NextItem Store.send median ms \(String(format: "%.2f", Self.median(storeNexts)))
      """
    Attachment.record(report, named: "baseline-1801.txt")
    #expect(decode < 2_000, "a decode past two seconds is a regression, not noise")
  }
}
