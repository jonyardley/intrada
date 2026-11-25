import SharedTypes
import Testing

@testable import Intrada

/// The profile sheet's unsaved icon through the real bridge (#846, #2519).
@MainActor
struct ProfileDraftBridgeTests {

  @Test func theDraftsIconComesBackAndEndsWithTheSheet() throws {
    let bridge = RowsBridge()

    let draft = Profile(name: "Jon", instrument: "Violin", iconChoice: .cello, colour: .coral)
    _ = try bridge.update(.profile(.draft(draft)))

    #expect(try bridge.rendered().profile.draftIcon == .cello)

    _ = try bridge.update(.profile(.draftEnded))

    #expect(try bridge.rendered().profile.draftIcon == nil)
  }
}
