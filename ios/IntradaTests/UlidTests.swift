import Testing

@testable import Intrada

/// Pins the encoder against the ulid spec.
/// `testRealBridgeAcceptsAUlidTheShellMinted` pins it against Rust's parser.
struct UlidTests {
  private static let crockford = Set("0123456789ABCDEFGHJKMNPQRSTVWXYZ")

  @Test func encodesTheSpecsWorkedExample() {
    // github.com/ulid/spec: 1469918176385 is "01ARYZ6S41" in the time portion.
    let ulid = Ulid.generate(millisecondsSinceEpoch: 1_469_918_176_385, randomByte: { 0 })

    #expect(ulid == "01ARYZ6S410000000000000000")
  }

  @Test func isTwentySixCrockfordCharacters() {
    let ulid = Ulid.generate()

    #expect(ulid.count == 26)
    #expect(ulid.allSatisfy { Self.crockford.contains($0) })
  }

  @Test func sortsByTheTimeItWasMinted() {
    let earlier = Ulid.generate(millisecondsSinceEpoch: 1_000_000_000_000, randomByte: { 255 })
    let later = Ulid.generate(millisecondsSinceEpoch: 1_000_000_000_001, randomByte: { 0 })

    #expect(earlier < later)
  }

  @Test func twoIdsMintedInTheSameMillisecondDiffer() {
    let ids = Set((0..<64).map { _ in Ulid.generate(millisecondsSinceEpoch: 1_700_000_000_000) })

    #expect(ids.count == 64)
  }
}
