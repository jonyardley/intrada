import PhotosUI
import SwiftUI

/// Camera + photo library picker with client-side compression and upload to API.
///
/// Used within LessonCaptureView and LessonDetailView (edit mode).
/// Photos are compressed to 2048px longest edge, JPEG 80% before upload.
/// Upload goes directly to POST /api/lessons/:id/photos (outside Crux).
struct PhotoCaptureView: View {
    let lessonId: String
    let onPhotoUploaded: () -> Void

    @Environment(IntradaCore.self) private var core
    @State private var selectedItems: [PhotosPickerItem] = []
    @State private var isUploading = false
    @State private var uploadError: String?
    @State private var showCamera = false

    var body: some View {
        VStack(alignment: .leading, spacing: Spacing.cardCompact) {
            Text("Photos")
                .font(.subheadline)
                .fontWeight(.medium)
                .foregroundStyle(Color.textLabel)

            HStack(spacing: Spacing.cardCompact) {
                // Camera button (iOS only, not available on simulator sometimes)
                if UIImagePickerController.isSourceTypeAvailable(.camera) {
                    Button {
                        showCamera = true
                    } label: {
                        Label("Camera", systemImage: "camera")
                            .font(.subheadline)
                    }
                    .buttonStyle(SecondaryButtonStyle())
                }

                // Photo library picker
                PhotosPicker(
                    selection: $selectedItems,
                    maxSelectionCount: 5,
                    matching: .images
                ) {
                    Label("Photo Library", systemImage: "photo.on.rectangle")
                        .font(.subheadline)
                }
                .buttonStyle(SecondaryButtonStyle())

                if isUploading {
                    ProgressView()
                        .controlSize(.small)
                }
            }

            if let error = uploadError {
                Text(error)
                    .font(.caption)
                    .foregroundStyle(Color.dangerText)
            }
        }
        .onChange(of: selectedItems) { _, newItems in
            Task {
                for item in newItems {
                    await uploadPhotoItem(item)
                }
                selectedItems = []
            }
        }
        .fullScreenCover(isPresented: $showCamera) {
            CameraView { image in
                Task {
                    await uploadImage(image)
                }
            }
        }
    }

    private func uploadPhotoItem(_ item: PhotosPickerItem) async {
        guard let data = try? await item.loadTransferable(type: Data.self) else {
            uploadError = "Failed to load photo"
            return
        }
        guard let uiImage = UIImage(data: data) else {
            uploadError = "Invalid image data"
            return
        }
        await uploadImage(uiImage)
    }

    private func uploadImage(_ image: UIImage) async {
        isUploading = true
        uploadError = nil
        defer { isUploading = false }

        // Compress to 2048px max edge
        let compressed = compressImage(image, maxEdge: 2048, quality: 0.8)
        guard let jpegData = compressed else {
            uploadError = "Failed to compress photo"
            return
        }

        // Upload directly to API (outside Crux)
        do {
            try await uploadToAPI(jpegData: jpegData)
            onPhotoUploaded()
        } catch {
            uploadError = "Upload failed: \(error.localizedDescription)"
        }
    }

    private func compressImage(_ image: UIImage, maxEdge: CGFloat, quality: CGFloat) -> Data? {
        let size = image.size
        let longestEdge = max(size.width, size.height)

        if longestEdge <= maxEdge {
            return image.jpegData(compressionQuality: quality)
        }

        let scale = maxEdge / longestEdge
        let newSize = CGSize(width: size.width * scale, height: size.height * scale)

        UIGraphicsBeginImageContextWithOptions(newSize, true, 1.0)
        image.draw(in: CGRect(origin: .zero, size: newSize))
        let resized = UIGraphicsGetImageFromCurrentImageContext()
        UIGraphicsEndImageContext()

        return resized?.jpegData(compressionQuality: quality)
    }

    private func uploadToAPI(jpegData: Data) async throws {
        let token = try await core.getAuthToken()
        let url = URL(string: "\(Config.apiBaseURL)/api/lessons/\(lessonId)/photos")!

        var request = URLRequest(url: url)
        request.httpMethod = "POST"

        let boundary = UUID().uuidString
        request.setValue("multipart/form-data; boundary=\(boundary)", forHTTPHeaderField: "Content-Type")
        if let token {
            request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }

        var body = Data()
        body.append("--\(boundary)\r\n".data(using: .utf8)!)
        body.append("Content-Disposition: form-data; name=\"photo\"; filename=\"photo.jpg\"\r\n".data(using: .utf8)!)
        body.append("Content-Type: image/jpeg\r\n\r\n".data(using: .utf8)!)
        body.append(jpegData)
        body.append("\r\n--\(boundary)--\r\n".data(using: .utf8)!)
        request.httpBody = body

        let (_, response) = try await URLSession.shared.data(for: request)
        guard let httpResponse = response as? HTTPURLResponse,
              (200...299).contains(httpResponse.statusCode) else {
            throw URLError(.badServerResponse)
        }
    }
}

// MARK: - Camera UIKit Bridge

struct CameraView: UIViewControllerRepresentable {
    let onCapture: (UIImage) -> Void
    @Environment(\.dismiss) private var dismiss

    func makeUIViewController(context: Context) -> UIImagePickerController {
        let picker = UIImagePickerController()
        picker.sourceType = .camera
        picker.delegate = context.coordinator
        return picker
    }

    func updateUIViewController(_ uiViewController: UIImagePickerController, context: Context) {}

    func makeCoordinator() -> Coordinator {
        Coordinator(onCapture: onCapture, dismiss: dismiss)
    }

    class Coordinator: NSObject, UIImagePickerControllerDelegate, UINavigationControllerDelegate {
        let onCapture: (UIImage) -> Void
        let dismiss: DismissAction

        init(onCapture: @escaping (UIImage) -> Void, dismiss: DismissAction) {
            self.onCapture = onCapture
            self.dismiss = dismiss
        }

        func imagePickerController(
            _ picker: UIImagePickerController,
            didFinishPickingMediaWithInfo info: [UIImagePickerController.InfoKey: Any]
        ) {
            if let image = info[.originalImage] as? UIImage {
                onCapture(image)
            }
            dismiss()
        }

        func imagePickerControllerDidCancel(_ picker: UIImagePickerController) {
            dismiss()
        }
    }
}

// MARK: - Secondary Button Style

private struct SecondaryButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .padding(.horizontal, Spacing.cardCompact)
            .padding(.vertical, 8)
            .background(Color.surfaceInput)
            .cornerRadius(DesignRadius.input)
            .overlay(
                RoundedRectangle(cornerRadius: DesignRadius.input)
                    .stroke(Color.borderInput, lineWidth: 1)
            )
            .foregroundStyle(Color.textSecondary)
            .opacity(configuration.isPressed ? 0.7 : 1.0)
    }
}
