# Sketch kit

The parts a new screen is sketched from in Claude Design, on the real look of
the apps. It is a copy of the code, never the source: the SwiftUI and Compose
components in `design/components/` stay the kit the apps are built from (T38
in [the design principles](../../docs/design-principles.md)). The folder
mirrors the design system's `project/` folder, so a refresh is a copy.

## What is here

| Path | What it is | Made by |
|---|---|---|
| `tokens.json` | Every colour, type style, spacing, radius, shadow, opacity and icon size, in the design system's token list format | `just sketch-kit`, from `Theme.swift` |
| `token-notes.json` | The usage note shown beside each token | By hand; the recipe fails when a token has no note or a note has no token |
| `icons/*.svg` | The app's icons, in ink | `just sketch-kit`, from Android's `ic_*.xml` drawables |
| `icons.json` | Each SF Symbol the iPhone app draws, and the kit icon that stands in for it | By hand; the recipe fails when a mapped icon is missing |
| `components/kit.js` | The five web copies, as React components | By hand |
| `components/bundle.js` | `kit.js` with the icons inlined, as the design system loads it | `just sketch-kit` |
| `components/bundle.css` | The components' styles, from tokens only | By hand |
| `components/index.d.ts` | Each component's props | By hand |
| `components/<Name>/preview.html` | A live preview of each component | By hand |

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

Two looks follow decisions rather than today's code (Jon, 15 October 2026):

- **Cards have the lift**, as T24 decided. Neither app draws it yet; #2638
  restores it.
- **The tab bar is drawn as it looks on a real iPhone**, an opaque bar in
  `tabBarFill`. The white bar in the iPhone snapshot is a test artefact (#2607).

The library row draws the whole iPhone row on both platforms. Android shows
less today (#2606, #2679).

## Phone frames

`ScreenScaffold` takes `device: "ios"` or `"android"` and then draws the whole
phone: the status bar, the screen, and the tab bar when `tab` names one. The
sizes are the devices the snapshot tests use.

| | iPhone 13 | Pixel 7 |
|---|---|---|
| Screen | 390 by 844 points | 412 by 915 dp |
| Status bar | 47 | 24 |
| Navigation bar | 44 on a pushed screen, none at a tab's root | none |
| Tab bar | 49, then 34 for the home indicator | 56 per tab plus a 1 dp rule, then 24 for gesture navigation |

```js
const { ScreenScaffold, LibraryItemCard, MarkerButton } = window.Intrada;

<ScreenScaffold device="ios" tab="library" title="Library" subtitle="8 pieces · 4 exercises" addLabel="Add item">
  <LibraryItemCard kind="piece" title="Clair de lune" subtitle="Debussy" score={7} />
</ScreenScaffold>
```

## Icons

The kit's icons are Android's drawables, which draw the same job as each SF
Symbol. A symbol mapped to `null` in `icons.json` has no Android drawable yet:
a sketch names it in words. SF Symbols themselves cannot be shipped as files.
The icons in `icons/` are drawn in `ink`; inside the components they take the
text colour.
