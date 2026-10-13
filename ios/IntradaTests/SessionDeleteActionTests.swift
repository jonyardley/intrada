import IntradaCoreFFI
import SharedTypes
import SwiftUI
import UIKit
import XCTest

@testable import Intrada

/// The delete and its Undo reach the core from the controls a musician uses:
/// each test drives the control through its accessibility element, as
/// VoiceOver does, in a real window (#2581). XCTest, not Swift Testing: run
/// after the XCTest suites, a Swift Testing case never saw SwiftUI fill in a
/// scroll view's elements.
@MainActor
final class SessionDeleteActionTests: XCTestCase {
  func testDeleteButtonSendsDeleteSaved() throws {
    let (store, bridge) = Self.store()
    let window = Self.mount(
      PracticeSessionDetailScreen(session: .previewCompleted), store: store)
    defer { Self.unmount(window) }

    let button = try XCTUnwrap(Self.element(in: window, identifier: "sessionDetail.delete"))
    XCTAssertTrue(button.accessibilityActivate())

    XCTAssertEqual(bridge.deleted, [PracticeSessionView.previewCompleted.id])
  }

  func testHistoryCardSendsDeleteSaved() throws {
    let (store, bridge) = Self.store()
    let window = Self.mount(
      PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate), store: store)
    defer { Self.unmount(window) }

    let card = try XCTUnwrap(Self.element(in: window, identifier: "practice.sessionCard"))
    let delete = try XCTUnwrap(
      card.accessibilityCustomActions?.first { $0.name == "Delete session" })
    XCTAssertEqual(delete.actionHandler?(delete), true)

    XCTAssertEqual(bridge.deleted, [PracticeSessionView.previewCompleted.id])
  }

  func testUndoBannerSendsUndoDelete() throws {
    let (store, bridge) = Self.store(undoDelete: "Session deleted")
    let window = Self.mount(AppBannerStack(), store: store)
    defer { Self.unmount(window) }

    let banner = try XCTUnwrap(Self.element(in: window, label: "Note, Session deleted"))
    let undo = try XCTUnwrap(banner.accessibilityCustomActions?.first { $0.name == "Undo" })
    XCTAssertEqual(undo.actionHandler?(undo), true)

    XCTAssertEqual(bridge.undone, 1)
  }

  // ── Harness ──

  private static func store(undoDelete: String? = nil) -> (Store, RecordingBridge) {
    let preview = PreviewBridge(
      sessions: [.previewCompleted, .previewEndedEarly], practiceWeeks: [.previewWeek],
      lastPractised: .previewYesterday)
    let bridge = RecordingBridge(inner: preview, undoDelete: undoDelete)
    let store = Store(bridge: bridge)
    store.receive(preview.sections)
    return (store, bridge)
  }

  /// SwiftUI builds its accessibility tree only while an assistive technology
  /// is on; this is the switch VoiceOver and the UI test runner flip. Turned
  /// off again on unmount so no other suite renders with it on.
  private static func setAccessibility(_ on: Bool) {
    guard let handle = dlopen("/usr/lib/libAccessibility.dylib", RTLD_NOW),
      let symbol = dlsym(handle, "_AXSApplicationAccessibilitySetEnabled")
    else { return }
    typealias SetEnabled = @convention(c) (Bool) -> Void
    unsafeBitCast(symbol, to: SetEnabled.self)(on)
  }

  private static func unmount(_ window: UIWindow) {
    window.rootViewController = nil
    setAccessibility(false)
  }

  private static func mount(_ view: some View, store: Store) -> UIWindow {
    IntradaFonts.register()
    setAccessibility(true)
    let controller = UIHostingController(
      rootView: NavigationStack { view }
        .environment(store)
        .environment(\.locale, Locale(identifier: "en_US"))
        .environment(\.calendar, PreviewCalendar.utc)
        .environment(\.intradaMotionDisabled, true))
    let window = UIWindow(frame: CGRect(x: 0, y: 0, width: 390, height: 2000))
    window.rootViewController = controller
    window.isHidden = false
    controller.view.layoutIfNeeded()
    return window
  }

  private static func element(in window: UIWindow, identifier: String) -> NSObject? {
    // SwiftUI's elements answer by key, not through `UIAccessibilityIdentification`.
    element(in: window) { ($0.value(forKey: "accessibilityIdentifier") as? String) == identifier }
  }

  private static func element(in window: UIWindow, label: String) -> NSObject? {
    element(in: window) { $0.accessibilityLabel == label }
  }

  /// SwiftUI fills the tree in after layout, later under a loaded run, so wait
  /// for the element rather than for a fixed time.
  private static func element(
    in window: UIWindow, matching match: (NSObject) -> Bool
  ) -> NSObject? {
    let deadline = Date().addingTimeInterval(5)
    repeat {
      if let found = accessibilityTree(window).first(where: match) { return found }
      RunLoop.current.run(until: Date().addingTimeInterval(0.05))
    } while Date() < deadline
    let labels = accessibilityTree(window).compactMap { $0 is UIView ? nil : $0.accessibilityLabel }
    XCTFail("No such element among: \(labels.joined(separator: "; "))")
    return nil
  }

  private static func accessibilityTree(_ root: NSObject) -> [NSObject] {
    var found: [NSObject] = []
    var seen: Set<ObjectIdentifier> = []
    var queue: [NSObject] = [root]
    while !queue.isEmpty {
      let node = queue.removeFirst()
      guard seen.insert(ObjectIdentifier(node)).inserted else { continue }
      found.append(node)
      queue.append(contentsOf: (node.accessibilityElements as? [NSObject]) ?? [])
      let count = node.accessibilityElementCount()
      if count > 0, count != NSNotFound {
        for index in 0..<count {
          if let child = node.accessibilityElement(at: index) as? NSObject { queue.append(child) }
        }
      }
      if let view = node as? UIView { queue.append(contentsOf: view.subviews) }
    }
    return found
  }
}

/// The preview bridge's screens, with every event the shell sends kept.
private final class RecordingBridge: CoreBridge {
  private let inner: CoreBridge
  private let undoDelete: String?
  private(set) var events: [Event] = []

  init(inner: CoreBridge, undoDelete: String?) {
    self.inner = inner
    self.undoDelete = undoDelete
  }

  var deleted: [String] {
    events.compactMap {
      if case .session(.deleteSaved(let id, _)) = $0 { id } else { nil }
    }
  }

  var undone: Int {
    events.filter {
      if case .session(.undoDelete) = $0 { true } else { false }
    }.count
  }

  func update(_ event: Event) throws -> [Request] {
    events.append(event)
    return try inner.update(event)
  }

  func resolve(_ id: UInt32, persistenceOutput: PersistenceOutput) throws -> [Request] {
    try inner.resolve(id, persistenceOutput: persistenceOutput)
  }

  func resolve(_ id: UInt32, recognitionOutput: RecognitionOutput) throws -> [Request] {
    try inner.resolve(id, recognitionOutput: recognitionOutput)
  }

  func resolveEmpty(_ id: UInt32) throws -> [Request] { try inner.resolveEmpty(id) }

  func view() throws -> ViewModel {
    var model = try inner.view()
    model.undoDelete = undoDelete
    return model
  }
}
