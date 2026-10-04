import SharedTypes
import Testing

@testable import Intrada

/// The keys sheet's taps over a set of keys, against the core's own wheel
/// (#2247). Ring 0 is C and A minor; ring 6 is G flat, spelt F sharp on tap two.
struct KeySetTests {
  private static var cMajor: Key { Key(letter: .c, accidental: .natural, mode: .major) }
  private static var aMinor: Key { Key(letter: .a, accidental: .natural, mode: .minor) }

  @Test("a tap on an empty spoke adds its key after the others")
  func tapAdds() {
    let keys = KeySet.tap([Self.aMinor], ring: 0, mode: .major)
    #expect(keys == [Self.aMinor, Self.cMajor])
  }

  @Test("a tap on a chosen spoke with one spelling removes it")
  func tapRemoves() {
    #expect(KeySet.tap([Self.cMajor, Self.aMinor], ring: 0, mode: .major) == [Self.aMinor])
  }

  @Test("a spoke with two spellings switches on the second tap and goes on the third")
  func tapSwitchesThenRemoves() throws {
    let added = KeySet.tap([], ring: 6, mode: .major)
    let first = try #require(KeySet.chosenSpelling(in: added, ring: 6, mode: .major))
    let switched = KeySet.tap(added, ring: 6, mode: .major)
    let second = try #require(KeySet.chosenSpelling(in: switched, ring: 6, mode: .major))
    #expect(first != second)
    #expect(switched.count == 1)
    #expect(KeySet.tap(switched, ring: 6, mode: .major).isEmpty)
  }

  @Test("the major and minor of one spoke are separate keys")
  func modesAreSeparate() {
    let keys = KeySet.tap([Self.cMajor], ring: 0, mode: .minor)
    #expect(keys == [Self.cMajor, Self.aMinor])
  }

  @Test("all major fills the twelve majors and keeps chosen minors and spellings")
  func addingAllMajors() {
    let flipped = KeySet.tap(KeySet.tap([], ring: 6, mode: .major), ring: 6, mode: .major)
    let start = flipped + [Self.aMinor]
    let keys = KeySet.addingAll(.major, to: start)
    #expect(keys.count == 13)
    #expect(Array(keys.prefix(2)) == start)
    #expect(
      KeySet.chosenSpelling(in: keys, ring: 6, mode: .major)
        == KeySet.chosenSpelling(in: flipped, ring: 6, mode: .major))
    #expect((0..<12).allSatisfy { KeySet.chosenSpelling(in: keys, ring: $0, mode: .major) != nil })
  }

  @Test("all major on an empty set is the circle the old preset gave")
  func addingAllToNothing() {
    #expect(KeySet.addingAll(.major, to: []) == KeyHelper.circle(.major))
  }
}
