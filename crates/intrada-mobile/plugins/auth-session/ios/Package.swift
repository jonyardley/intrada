// swift-tools-version:5.3
import PackageDescription

let package = Package(
  name: "tauri-plugin-auth-session",
  platforms: [
    .iOS(.v13)
  ],
  products: [
    .library(
      name: "tauri-plugin-auth-session",
      type: .static,
      targets: ["tauri-plugin-auth-session"])
  ],
  dependencies: [
    .package(name: "Tauri", path: "../.tauri/tauri-api")
  ],
  targets: [
    .target(
      name: "tauri-plugin-auth-session",
      dependencies: [
        .byName(name: "Tauri")
      ],
      path: "Sources/AuthSessionPlugin")
  ]
)
