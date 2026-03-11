import Foundation

/// App-wide configuration constants.
///
/// In production these would be injected via build settings or a .xcconfig file.
/// For development, defaults are provided here.
enum Config {
    /// Clerk publishable key — baked into the app at build time.
    ///
    /// Set via CLERK_PUBLISHABLE_KEY build setting, Xcode scheme environment variable,
    /// or replace the fallback below with your actual key from https://dashboard.clerk.com
    ///
    /// Publishable keys look like: pk_test_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
    static let clerkPublishableKey: String = {
        if let key = ProcessInfo.processInfo.environment["CLERK_PUBLISHABLE_KEY"], !key.isEmpty {
            return key
        }
        // ⚠️ Replace with your actual Clerk publishable key for local development.
        // Get it from: Clerk Dashboard → API Keys → Publishable key
        return "pk_live_Y2xlcmsubXlpbnRyYWRhLmNvbSQ"
    }()

    /// Whether the Clerk publishable key looks valid (starts with pk_test_ or pk_live_
    /// and has content after the prefix).
    static var isClerkConfigured: Bool {
        let key = clerkPublishableKey
        return (key.hasPrefix("pk_test_") || key.hasPrefix("pk_live_"))
            && key.count > 10
    }

    /// Base URL for the Intrada REST API.
    static let apiBaseURL: String = {
        if let url = ProcessInfo.processInfo.environment["INTRADA_API_URL"], !url.isEmpty {
            return url
        }
        return "https://intrada-api.fly.dev"
    }()
}
