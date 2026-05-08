// swift-tools-version:5.9
// IntradaActivityShared — type definitions shared between the
// tauri-plugin-live-activity Swift target and the IntradaLiveActivity
// widget extension. Live Activities require the SAME `ActivityAttributes`
// type identity on both sides — Swift's type system enforces this, so
// duplicating the struct in two targets won't work; they have to share
// a single declaration via a shared module.
//
// Spec: ../../../../specs/live-activity-plugin.md (Phase C, #474).

import PackageDescription

let package = Package(
  name: "IntradaActivityShared",
  platforms: [
    // ActivityKit shipped in iOS 16.1. The `ActivityAttributes`
    // conformance on `IntradaActivityAttributes` is `@available(iOS
    // 16.1, *)` gated, so the platform target needs to match.
    .iOS("16.1")
  ],
  products: [
    .library(
      name: "IntradaActivityShared",
      type: .static,
      targets: ["IntradaActivityShared"])
  ],
  targets: [
    .target(
      name: "IntradaActivityShared",
      path: "Sources/IntradaActivityShared")
  ]
)
