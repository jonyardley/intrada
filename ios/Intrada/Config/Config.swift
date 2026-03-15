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

    /// Base URL for the Intrada REST API.
    static let apiBaseURL: String = {
        if let url = ProcessInfo.processInfo.environment["INTRADA_API_URL"], !url.isEmpty {
            return url
        }
        return "https://intrada-api.fly.dev"
    }()
}
