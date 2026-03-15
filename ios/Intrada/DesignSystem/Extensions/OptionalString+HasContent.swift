import Foundation

extension Optional where Wrapped == String {
    /// True when the string is non-nil and non-empty.
    /// Useful for form error validation: `error.hasContent`.
    var hasContent: Bool {
        guard let value = self else { return false }
        return !value.isEmpty
    }
}
