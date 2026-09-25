import SharedTypes
import SwiftUI

struct PracticeSplitView: View {
  @Environment(Store.self) private var store
  @Environment(\.horizontalSizeClass) private var sizeClass
  private let referenceDate: Date
  @State private var selectedId: String?

  init(referenceDate: Date = Date()) {
    self.referenceDate = referenceDate
  }

  #if DEBUG
    /// Preview/snapshot seed: render with a session already selected.
    init(referenceDate: Date, previewSelection: String?) {
      self.referenceDate = referenceDate
      _selectedId = State(initialValue: previewSelection)
    }
  #endif

  private var selectedSession: PracticeSessionView? {
    selectedId.flatMap { id in store.sessionHistory.first { $0.id == id } }
  }

  var body: some View {
    if sizeClass == .regular {
      ListDetailSplit {
        PracticeScreen(referenceDate: referenceDate, selection: $selectedId)
      } detail: {
        if let selectedSession {
          PracticeSessionDetailScreen(session: selectedSession)
        } else {
          SplitDetailPlaceholder(message: "Select a session to see how it went.")
        }
      }
    } else {
      NavigationStack {
        PracticeScreen(referenceDate: referenceDate).navigationBarHiddenAtRoot()
      }
    }
  }
}
