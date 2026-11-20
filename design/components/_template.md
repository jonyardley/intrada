---
name: Card
status: stable
ios_symbol: cardSurface, cardShadow
ios_file: ios/Intrada/DesignSystem/CardSurface.swift, ios/Intrada/DesignSystem/CardShadow.swift
ios_snapshots: ComponentSnapshotTests/testCard.1.png
decisions: T24, T36
---

<!--
Copy this file to design/components/<Name>/README.md and add the name to the
index. Keys sit one to a line so scripts/check-components.sh can read them:

- status: stable, proposed or deprecated.
- ios_symbol and ios_file: comma separated; each symbol must be declared in one of the files, and the files sit under ios/.
- ios_snapshots: paths under ios/IntradaTests/__Snapshots__/.

Delete any section below that has nothing to say, rather than filling it.
The page is the lasting reference: its purpose, the one look and its rules.
No caller counts or lists of where it is used, which go stale with every
change; a known gap is one line naming the issue that settles it. Name the
tokens in Content rather than listing parts.
-->

# Card

One sentence on what it is for. The Claude Design page shows this as the summary.

## When to use

## When not to use

Name the component to use instead.

## Variants

One line each: "use for ...".

## Behaviour

Interaction, motion and haptics. A known gap is one line naming the issue that settles it.

## Content

The tokens it is drawn with, wording rules and how long the text can be. States the picture does not show, such as disabled or error.

## Accessibility

Label, Dynamic Type, touch size.

## Do and don't

Pairs, each with a snapshot.

## Usage

```swift
```

## Related
