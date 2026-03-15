import SwiftUI

/// Toast notification variant matching the web's `ToastVariant` enum.
enum ToastVariant {
    case info
    case success
    case warning
    case danger

    var iconName: String {
        switch self {
        case .info: "info.circle.fill"
        case .success: "checkmark.circle.fill"
        case .warning: "exclamationmark.triangle.fill"
        case .danger: "xmark.circle.fill"
        }
    }

    var borderColor: Color {
        switch self {
        case .info: .info
        case .success: .success
        case .warning: .warning
        case .danger: .danger
        }
    }

    var backgroundColor: Color {
        switch self {
        case .info: .infoSurface
        case .success: .successSurface
        case .warning: .warningSurface
        case .danger: .dangerSurface
        }
    }

    var textColor: Color {
        switch self {
        case .info: .infoText
        case .success: .successText
        case .warning: .warningText
        case .danger: .dangerText
        }
    }
}

/// App-wide toast notification manager.
///
/// Inject via `.environment(toastManager)` and read in views to trigger toasts.
///
///     // In a view:
///     @Environment(ToastManager.self) private var toast
///     toast.show("Item saved", variant: .success)
@Observable
@MainActor
final class ToastManager {

    private(set) var message: String = ""
    private(set) var variant: ToastVariant = .info
    private(set) var isShowing: Bool = false

    private var dismissTask: Task<Void, Never>?

    /// Show a toast notification that auto-dismisses after 3 seconds.
    func show(_ message: String, variant: ToastVariant = .info) {
        dismissTask?.cancel()

        self.message = message
        self.variant = variant

        withAnimation(.easeOut(duration: 0.25)) {
            self.isShowing = true
        }

        dismissTask = Task {
            try? await Task.sleep(for: .seconds(3))
            guard !Task.isCancelled else { return }
            dismiss()
        }
    }

    /// Manually dismiss the toast.
    func dismiss() {
        withAnimation(.easeIn(duration: 0.2)) {
            isShowing = false
        }
    }
}
