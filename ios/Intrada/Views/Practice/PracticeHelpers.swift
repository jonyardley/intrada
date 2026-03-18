import Foundation

// MARK: - Shared helpers for Practice views

/// Estimate total session duration from setlist entries.
///
/// Uses explicit `plannedDurationSecs` when set; falls back to
/// 5 minutes per entry when no durations are configured.
func estimatedTotalMinutes(for entries: [SetlistEntryView]) -> Int {
    let totalSecs = entries.compactMap { (e: SetlistEntryView) -> UInt32? in
        e.plannedDurationSecs
    }.reduce(0, +)
    return totalSecs > 0 ? Int(totalSecs) / 60 : entries.count * 5
}
