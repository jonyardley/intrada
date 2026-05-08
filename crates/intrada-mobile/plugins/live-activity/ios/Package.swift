// swift-tools-version:5.9
import PackageDescription

let package = Package(
  name: "tauri-plugin-live-activity",
  platforms: [
    // Plugin target compiles down to iOS 13 so it can ship alongside
    // the rest of the app (which targets 14.0). ActivityKit calls are
    // gated with @available(iOS 16.1, *) at the call sites.
    .iOS(.v13)
  ],
  products: [
    .library(
      name: "tauri-plugin-live-activity",
      type: .static,
      targets: ["tauri-plugin-live-activity"])
  ],
  dependencies: [
    // Tauri SwiftPM umbrella — generated under the host Xcode project's
    // `gen/apple/.tauri/tauri-api/` when `cargo tauri ios init` runs.
    .package(name: "Tauri", path: "../.tauri/tauri-api"),
    // Shared ActivityAttributes type — same identity must appear here
    // and on the widget extension. See
    // crates/intrada-mobile/shared/IntradaActivityShared/.
    .package(name: "IntradaActivityShared", path: "../../../shared/IntradaActivityShared"),
  ],
  targets: [
    .target(
      name: "tauri-plugin-live-activity",
      dependencies: [
        .byName(name: "Tauri"),
        .byName(name: "IntradaActivityShared"),
      ],
      path: "Sources/LiveActivityPlugin")
  ]
)
