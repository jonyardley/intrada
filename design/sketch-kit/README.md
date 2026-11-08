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
| `components/tokens.css` | `tokens.json` as CSS: a variable per token, the font faces and a class per type style | `just sketch-kit` |
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

Two looks follow decisions rather than the code (Jon, 15 October 2026):

- **Cards have the lift**, as T24 decided. The kit draws it by default
  (`lift={false}` turns it off); neither app draws it until #2638 lands.
- **The tab bar is drawn as it looks on a real iPhone**, an opaque bar in
  `tabBarFill`. The white bar in the iPhone snapshot is a test artefact (#2607).
  Its labels use `smallMedium`, the smallest type style, though the phone draws
  them a little smaller in the system font: the kit keeps to the theme's type.

The library row draws the whole iPhone row on both platforms. Android shows
less until #2606 and #2679 land.

The title's highlighter swipe is drawn on iPhone only, as in the apps:
Android's `ScreenScaffold` has none until #2605.

A screen's top-level list, such as the Library's items or the Practice home's
sessions, is one card per entry, `spacing-cardCompact` (12) apart. A list
inside a detail card, such as a piece's linked exercises, sections or
variations, or the items in a session being built, is rows in that one card
with hairlines between them, as are a form's fields.

## Phone frames

`ScreenScaffold` takes `device: "ios"` or `"android"` and then draws the whole
phone: the status bar, the screen, and the tab bar when `tab` names one. The
sizes are the devices the snapshot tests use.

| | iPhone 13 | Pixel 7 |
|---|---|---|
| Screen | 390 by 844 points | 411 by 914 dp |
| Status bar | 47 | 24 |
| Navigation bar | 44 on a pushed screen, none at a tab's root | none |
| Tab bar | 49, then 34 for the home indicator | 56 per tab plus a 1 dp rule, then 24 for gesture navigation |

```js
const { ScreenScaffold, LibraryItemCard, MarkerButton } = window.Intrada;

<ScreenScaffold device="ios" tab="library" title="Library" subtitle="8 pieces · 4 exercises" addLabel="Add item">
  <LibraryItemCard kind="piece" title="Clair de lune" subtitle="Debussy" score={7} />
</ScreenScaffold>
```

### Pushed screens

A screen opened from another one sits in its parent's tab.

- **iPhone:** pass `pushed` and the parent's `tab`. The tab bar stays, with
  the parent tab selected, and a back chevron sits above the title.
- **Android:** leave `tab` off. The app draws no tab bar on a pushed screen,
  and the kit draws no back arrow, because the app has none until #2619.

```js
<ScreenScaffold device="ios" tab="practice" pushed title="Session" />
<ScreenScaffold device="android" title="Session" />
```

## Consuming

How a Claude Design canvas uses the kit. The canvas type's own reference on
design system components governs how files reach a canvas; this section says
what the kit needs.

1. **Make sure `window.React` and `window.ReactDOM` are React 18.** The kit
   reads `window.React.createElement`, so an ES module import is not enough.
   Only when the canvas does not already provide React 18, load React 18.3.1
   and ReactDOM 18.3.1 as UMD scripts; never load a second copy.
2. **Link `tokens.css`, then `bundle.css`**, and load `bundle.js` after React.
   A canvas takes each as an uploaded file and links the address the upload
   returns. Upload the fonts in `project/fonts/` too and point the font faces
   in `tokens.css` at their uploaded addresses, or the text falls back to the
   system font.
3. **Take the components from `window.Intrada`.** Set the canvas page on
   `var(--paperTop)` in `var(--font-sans)`.

`tokens.css` defines general class names, such as `.body`, `.title`,
`.label`, `.button` and `.small`, and they apply to the whole canvas page, not
only the kit's components.

| Component | Props (`index.d.ts`) | Children |
|---|---|---|
| `ScreenScaffold` | `title`; optional `subtitle`, `device` (`"ios"` or `"android"`), `tab` (`"library"`, `"practice"`, `"routines"`, `"progress"`), `pushed`, `addLabel`, `actions`, `marker`, `noSwipe`, `reserveSubtitle` | The screen's content, under the header rule |
| `Card` | Optional `lift` (on unless `false`), `compact`, `style` | What the card holds |
| `LibraryItemCard` | `title`, `kind` (`"piece"` or `"exercise"`); optional `subtitle`, `keyName`, `tempo`, `priority`, `linkedExercises`, `keys`, `variations`, `tags`, `score` (0 to 10, or `null`), `showsMastery` | None |
| `MarkerButton` | `label`; optional `marker` | None |
| `TabBar` | Optional `device`, `selected` | None |

`marker` is one of `butter`, `coral`, `mint`, `sky`, `lavender`, `sage`,
`peach` or `powder`; butter by default.

The two React script tags are only for a canvas that does not already provide
React 18; leave them out when it does.

```html
<!-- Only when the canvas does not already provide React 18. -->
<script src="https://cdnjs.cloudflare.com/ajax/libs/react/18.3.1/umd/react.production.min.js"></script>
<script src="https://cdnjs.cloudflare.com/ajax/libs/react-dom/18.3.1/umd/react-dom.production.min.js"></script>
<link rel="stylesheet" href="TOKENS_CSS">
<link rel="stylesheet" href="BUNDLE_CSS">
<script src="BUNDLE_JS"></script>
<div id="root"></div>
<script>
  var h = React.createElement, I = window.Intrada;
  ReactDOM.createRoot(document.getElementById("root")).render(
    h(I.ScreenScaffold, { device: "ios", tab: "practice", title: "Practice" },
      h(I.Card, null, h("div", { className: "cardTitle" }, "Today")))
  );
</script>
```

`TOKENS_CSS`, `BUNDLE_CSS` and `BUNDLE_JS` stand for the addresses the
canvas's uploads return.

## Icons

The kit's icons are Android's drawables, which draw the same job as each SF
Symbol. A symbol mapped to `null` in `icons.json` has no Android drawable:
a sketch names it in words. SF Symbols themselves cannot be shipped as files.
The icons in `icons/` are drawn in `ink`; inside the components they take the
text colour.
