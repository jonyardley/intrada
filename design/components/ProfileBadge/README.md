---
name: ProfileBadge
status: stable
ios_symbol: ProfileBadge
ios_file: ios/Intrada/DesignSystem/ProfileBadge.swift
ios_snapshots: ComponentSnapshotTests/testProfileBadge.1.png
android_symbol: ProfileBadge
android_file: android/app/src/main/kotlin/com/intrada/android/ui/components/ProfileBadge.kt
android_snapshots: profile-badge.png
decisions: T24, T25, T28
---

# ProfileBadge

The musician's instrument on a disc of their highlighter: who this notebook belongs to, and the way into their profile.

## When to use

- The Practice header, where a tap opens the profile.
- The top of the profile screen and the first run, where the musician sees the instrument they chose.

## When not to use

- An item's type: use `TypeBadge`.
- A mark: use `ScoreRing`.
- Any other round icon button: the highlighter disc belongs to the musician's own identity and the one call to action.

## Variants

- **Bar** (`IntradaGlyph.bar`, 36): use in a header.
- **Tile** (`IntradaGlyph.tile`, 56, the default): use beside a page title, as on the first run.
- **Hero** (`IntradaGlyph.hero`, 88): use at the top of the profile.

## Behaviour

None of its own. Where it opens the profile, the button around it carries the press style and the label.

## Content

- The disc is the musician's marker colour; the instrument glyph is `onMarker`, at 62% of the disc.
- Before the profile has loaded, callers pass the `other` glyph.
- iPhone reads the marker from the environment; Android takes it as a parameter. The look is the same.

## Accessibility

- The glyph is read by the instrument's name on both apps.
- Where the badge is a button, the button carries the label ("Profile" in the Practice header) and a touch area of at least 44 points.
- Beside a title that already says what it is, as on the first run, hide the badge from VoiceOver and TalkBack.
- The badge is a fixed size and does not scale with Dynamic Type or font scale.

## Usage

```swift
ProfileBadge(icon: profile.icon, size: IntradaGlyph.hero)
```

```kotlin
ProfileBadge(profile.icon, IntradaColor.marker(colour), size = IntradaGlyph.hero)
```

## Related

`TypeBadge`, an item's type. `UpNextHero`, which sits below it on Practice.
