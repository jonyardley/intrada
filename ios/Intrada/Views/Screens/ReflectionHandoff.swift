import SharedTypes

enum ReflectionHandoff {
  /// A halted app has no core error to read, and a retry can never succeed (#2009).
  @MainActor static func refusalMessage(halted: Bool, error: String?) -> String {
    if halted { return Store.haltedMessage }
    return error ?? "Couldn't save. Try again."
  }
}
