# Sketch kit

The parts a new screen is sketched from in Claude Design, on the real look of
the apps. It is a copy of the code, never the source: the SwiftUI and Compose
components in `design/components/` stay the kit the apps are built from (T38
in [the design principles](../../docs/design-principles.md)). The folder
mirrors the design system's `project/` folder, so a refresh is a copy;
`design-system.md` becomes its `project/README.md`.

## What is here

| Path | What it is | Made by |
|---|---|---|
| `tokens.json` | Every colour, type style, spacing, radius, shadow, opacity and icon size, in the design system's token list format | `just sketch-kit`, from `Theme.swift` |
| `token-notes.json` | The usage note shown beside each token | By hand; the recipe fails when a token has no note or a note has no token |
| `icons/*.svg` | The app's icons, in ink | `just sketch-kit`, from Android's `ic_*.xml` drawables |
| `icons.json` | Each SF Symbol the iPhone app draws, and the kit icon that stands in for it | By hand; the recipe fails when a mapped icon is missing |
| `components/kit.js` | The five web copies, as React components | By hand |
| `components/bundle.js` | `kit.js` with the icons inlined, as the design system loads it | `just sketch-kit` |
| `components/tokens.css` | `tokens.json` as CSS: a variable per token, the font faces and a class per type style | `just sketch-kit` |
| `components/bundle.css` | The components' styles, from tokens only | By hand |
| `components/index.d.ts` | Each component's props | By hand |
| `components/<Name>/preview.html` | A live preview of each component | By hand |
| `design-system.md` | The design system's own readme: its look in brief, the component list and how a canvas uses the kit | By hand |

Never edit a generated file. Change the code or the hand-written source, then
run `just sketch-kit`. `just check` fails when a generated file is stale, or
when the kit types a colour or a font size or names a token `tokens.json`
lacks.

## The five components

Exactly five, the ones nearly every screen is made of. A sketch that needs
anything else draws it from the tokens and names the catalogue component it
stands for.

| Web copy | Copies, on iPhone | Copies, on Android |
|---|---|---|
| `ScreenScaffold` | `ios/Intrada/DesignSystem/ScreenScaffold.swift` | `android/app/src/main/kotlin/com/intrada/android/ui/ScreenScaffold.kt` |
| `Card` | `ios/Intrada/DesignSystem/CardSurface.swift`, `CardShadow.swift` | `ui/components/CardSurface.kt`, `CardShadow.kt` |
| `LibraryItemCard` | `ios/Intrada/Views/Components/LibraryItemCard.swift` | `ui/LibraryScreen.kt` |
| `MarkerButton` | `ios/Intrada/DesignSystem/MarkerButton.swift` | `ui/FirstRunScreen.kt` |
| `TabBar` | `ios/Intrada/Views/RootView.swift` | `ui/AppFrame.kt`, `ui/AppTab.kt` |

Android paths are under `android/app/src/main/kotlin/com/intrada/android/`.

## Sketching from the kit

How a canvas uses the kit, the looks that follow decisions rather than the
code, the phone frames, pushed screens and each component's props are in
[`design-system.md`](design-system.md). The refresh publishes that file as the
design system's own readme, where a sketching agent looks for them.

## Icons

The kit's icons are Android's drawables, which draw the same job as each SF
Symbol. A symbol mapped to `null` in `icons.json` has no Android drawable:
a sketch names it in words. SF Symbols themselves cannot be shipped as files.
The icons in `icons/` are drawn in `ink`; inside the components they take the
text colour.
