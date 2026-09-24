import Foundation
import SharedTypes

extension LibraryStore {
  // ── Shared codec ─────────────────────────────────────────────────────

  // Surface (don't silently default) a stored enum string we don't recognise,
  // e.g. an older binary reading a row a newer version wrote (#949).
  static let decodeContext = "LibraryStore decode"

  struct UnknownStoredEnum: Error, CustomStringConvertible {
    let kind: String
    let raw: String
    var description: String { "unknown \(kind) on decode: \"\(raw)\"" }
  }

  // A JSON column that will not decode is reported and reads back as empty; one
  // that will not encode throws, so the save fails instead of storing "[]" (#1117).
  struct StoredCodecError: Error, CustomStringConvertible {
    let field: String
    var description: String { "\(field) failed to decode" }
  }

  static func encodeJSON<T: Encodable>(_ value: T) throws -> String {
    String(decoding: try JSONEncoder().encode(value), as: UTF8.self)
  }

  static func decodeJSON<T: Decodable>(_ type: T.Type, from json: String, field: String)
    -> T?
  {
    guard let value = tryDecodeJSON(type, from: json) else {
      report(StoredCodecError(field: field), decodeContext)
      return nil
    }
    return value
  }

  static func tryDecodeJSON<T: Decodable>(_ type: T.Type, from json: String) -> T? {
    try? JSONDecoder().decode(type, from: Data(json.utf8))
  }

  /// A new core case fails to compile in `encode`, but `cases` is listed by
  /// hand: a case missing there decodes as unknown. Unchecked `Sendable`
  /// because the generated SharedTypes enums declare none; every field is a `let`.
  struct StoredEnum<Value: Equatable>: @unchecked Sendable {
    let kind: String
    let cases: [Value]
    let encode: (Value) -> String

    func decode(_ raw: String) -> Value? {
      if let match = cases.first(where: { encode($0) == raw }) { return match }
      report(UnknownStoredEnum(kind: kind, raw: raw), decodeContext)
      return nil
    }
  }
}
