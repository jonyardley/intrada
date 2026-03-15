import SwiftUI
import ClerkKit

@main
struct IntradaApp: App {

    @State private var core = IntradaCore()
    @State private var clerk: Clerk
    @State private var toastManager = ToastManager()

    init() {
        // Always configure Clerk — store the returned instance directly
        // instead of relying on Clerk.shared (which crashes if configure fails).
        let configured = Clerk.configure(publishableKey: Config.clerkPublishableKey)
        _clerk = State(initialValue: configured)
    }

    var body: some Scene {
        WindowGroup {
            ContentRouter()
                .toastOverlay()
                .environment(clerk)
                .environment(core)
                .environment(toastManager)
                .preferredColorScheme(.dark)
        }
    }
}

// MARK: - Content Router (auth gate)

private struct ContentRouter: View {
    @Environment(Clerk.self) private var clerk
    @Environment(IntradaCore.self) private var core

    var body: some View {
        ZStack {
            if clerk.user != nil {
                MainTabView()
                    .onAppear {
                        core.setClerk(clerk)
                        core.startApp()
                    }
            } else {
                SignInView()
            }
        }
    }
}

// MARK: - Sign In View

struct SignInView: View {
    @Environment(Clerk.self) private var clerk
    @State private var isLoading = false
    @State private var errorMessage: String?

    var body: some View {
        ZStack {
            Color.backgroundApp
                .ignoresSafeArea()

            VStack(spacing: 32) {
                VStack(spacing: 8) {
                    Image(systemName: "music.note.list")
                        .font(.system(size: 48))
                        .foregroundStyle(Color.accent)
                    Text("intrada")
                        .font(.heading(size: 34))
                        .fontWeight(.bold)
                        .foregroundStyle(Color.textPrimary)
                    Text("Your music practice companion")
                        .font(.subheadline)
                        .foregroundStyle(Color.textSecondary)
                }

                Button {
                    Task {
                        isLoading = true
                        defer { isLoading = false }
                        do {
                            _ = try await clerk.auth.signInWithOAuth(provider: .google)
                        } catch {
                            errorMessage = error.localizedDescription
                        }
                    }
                } label: {
                    HStack {
                        if isLoading {
                            ProgressView()
                                .tint(Color.textPrimary)
                        } else {
                            Image(systemName: "person.crop.circle.fill")
                        }
                        Text("Sign in with Google")
                    }
                    .font(.headline)
                    .foregroundStyle(Color.textPrimary)
                    .frame(maxWidth: .infinity)
                    .padding()
                    .background(Color.accent)
                    .clipShape(RoundedRectangle(cornerRadius: DesignRadius.button))
                }
                .disabled(isLoading)
                .padding(.horizontal, 40)

                if let errorMessage {
                    Text(errorMessage)
                        .font(.caption)
                        .foregroundStyle(Color.dangerText)
                        .padding(.horizontal, 40)
                }
            }
        }
    }
}
