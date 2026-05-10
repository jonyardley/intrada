// swift-tools-version:5.3
import PackageDescription

let package = Package(
  name: "tauri-plugin-background-audio",
  platforms: [
    .iOS(.v13)
  ],
  products: [
    .library(
      name: "tauri-plugin-background-audio",
      type: .static,
      targets: ["tauri-plugin-background-audio"])
  ],
  dependencies: [
    // The Tauri SwiftPM umbrella package is generated under the host
    // Xcode project's `gen/apple/.tauri/tauri-api/` when `cargo tauri ios
    // init` runs. Path is relative to this Package.swift after the host
    // app's local plugin path declaration.
    .package(name: "Tauri", path: "../.tauri/tauri-api")
  ],
  targets: [
    .target(
      name: "tauri-plugin-background-audio",
      dependencies: [
        .byName(name: "Tauri")
      ],
      path: "Sources/BackgroundAudioPlugin")
  ]
)
