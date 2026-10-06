import Foundation
import IntradaCoreFFI
import SharedTypes
import Testing

@testable import Intrada

// Blobs on the device are the only copy: a renamed key strands every value
// written under the old one.
@MainActor
struct DefaultsKeysTests {
  @Test func defaultsKeys() {
    #expect(Store.sortDefaultsKey == "intrada.library-sort.v\(librarySortBlobVersion())")
    #expect(Store.legacySortDefaultsKey == "intrada.library-sort")
    #expect(Store.legacySortMovesToKey == "intrada.library-sort.v1")
    #expect(Store.profileDefaultsKey == "intrada.profile.v\(profileBlobVersion())")
    #expect(Store.firstRunKey == "intrada.first-run.v\(firstRunBlobVersion())")
    #expect(Store.practiceDefaultsKey == "intrada.practice-defaults.v2")  // gitleaks:allow
    #expect(Store.sessionInProgressKey == "intrada.session-in-progress.v\(sessionBlobVersion())")
  }
}
