import SwiftUI

/// A modifier that presents a delete confirmation alert.
struct DeleteConfirmationModifier: ViewModifier {
    let title: String
    let message: String
    @Binding var isPresented: Bool
    let onDelete: () -> Void

    func body(content: Content) -> some View {
        content
            .alert(title, isPresented: $isPresented) {
                Button("Delete", role: .destructive, action: onDelete)
                Button("Cancel", role: .cancel) {}
            } message: {
                Text(message)
            }
    }
}

extension View {
    func deleteConfirmation(
        _ title: String,
        message: String,
        isPresented: Binding<Bool>,
        onDelete: @escaping () -> Void
    ) -> some View {
        modifier(DeleteConfirmationModifier(
            title: title,
            message: message,
            isPresented: isPresented,
            onDelete: onDelete
        ))
    }
}
