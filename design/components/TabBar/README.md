---
name: TabBar
status: stable
ios_symbol: RootView, applyTabBarAppearance
ios_file: ios/Intrada/Views/RootView.swift
ios_snapshots: RootShellSnapshotTests/testRootShell.1.png
android_symbol: TabBar, TabItem, AppTab
android_file: android/app/src/main/kotlin/com/intrada/android/ui/AppFrame.kt, android/app/src/main/kotlin/com/intrada/android/ui/AppTab.kt
android_snapshots: frame-library.png
decisions: T24, T36
---

# TabBar

The four tabs along the bottom of the app: Library, Practice, Routines and Progress.

## When to use

- Only at the app's root. There is one tab bar, and every top-level screen sits under it.

## When not to use

- Inside a screen, to switch between views of the same thing: use `SegmentedPills`.
- During a session: the player covers the whole screen and the tab bar goes away.
- Over a pushed screen on Android, such as a piece or the item form: the bar shows only on the four tab roots.

## Variants

None. The four tabs are fixed, in this order.

## Content

- Each tab is an icon over a one-word title in sentence case. The shells write the titles: `RootView` on iPhone, `AppTab` on Android.
- The bar sits on `tabBarFill` (#F3EFE8). The chosen tab is in `accent` (ink); the others are in `inkSecondary`.
- iPhone draws the system tab bar, so on iOS 26 the system's glass styles the items itself.
- The iPhone picture (`testRootShell`) is what the test records, not the look: it draws the tabs white on paper. #2607 (I4) checks a device and settles which is true.
- How Android marks the chosen tab, what tapping it again does and the bar's height are #2617.

## Accessibility

- Each tab is read as its title, with the tab role and whether it is selected.
- On Android each tab is at least 56dp tall and fills a quarter of the width.
- Switching tabs gives no haptic on either app; whether it should is #2676.

## Do and don't

- **Do** keep a tab's title to one word.
- **Don't** add a fifth tab or a badge without a design decision first.

## Usage

```swift
TabView(selection: $selectedTab) {
  LibrarySplitView()
    .tabItem { Label("Library", systemImage: "books.vertical") }
    .tag(AppTab.library)
}
.tint(IntradaColor.accent)
```

```kotlin
if (onTab) TabBar(tab ?: AppTab.LIBRARY, onSelect = { navController.select(it) })
```

## Related

`ScreenScaffold`, the screen each tab opens.
