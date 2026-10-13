---
name: Card
status: stable
ios_symbol: cardSurface, cardShadow
ios_file: ios/Intrada/DesignSystem/CardSurface.swift, ios/Intrada/DesignSystem/CardShadow.swift
ios_snapshots: ComponentSnapshotTests/testCardSurface.1.png
android_symbol: cardSurface
android_file: android/app/src/main/kotlin/com/intrada/android/ui/components/CardSurface.kt
android_snapshots: card-surface.png
decisions: T24, T35
---

<!--
Copy this file to design/components/<Name>/README.md and add the name to the
index. Keys sit one to a line so scripts/check-components.sh can read them:

- status: stable, proposed or deprecated.
- *_symbol and *_file: comma separated; each symbol must appear in one of the files.
- ios_snapshots: paths under ios/IntradaTests/__Snapshots__/.
- android_snapshots: paths under android/app/src/test/snapshots/.
- No Android version: drop the three android_ keys and write
  `not_on_android: <why, or the issue that builds it>`.

Delete any section below that has nothing to say, rather than filling it.
-->

# Card

One sentence on what it is for. The Claude Design page shows this as the summary.

## When to use

## When not to use

Name the component to use instead.

## Anatomy

The named parts.

## Variants

One line each: "use for ...".

## States

Default, pressed, disabled, error.

## Behaviour

Interaction, motion and haptics, and any difference between the apps.

## Content

Wording rules and how long the text can be.

## Accessibility

Label, Dynamic Type and font scale, touch size.

## Do and don't

Pairs, each with a snapshot.

## Usage

```swift
```

```kotlin
```

## Related

## Changes

| Date | Issue | What changed |
|---|---|---|
