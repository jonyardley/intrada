Intrada is a practice notebook for musicians, with an iPhone app. This is a published copy of the component catalogue in the repo (`design/components/` in jonyardley/intrada); the commit it was taken from is at the end of this page. The repo is the source: change a page there, never here (T37 in [the design principles](../../docs/design-principles.md)).

## Paper, ink and one highlighter

- Set every screen on `paperTop`, an off-white paper.
- Set text and anything you can press in `ink`. Interactive colour on paper is ink; there is no brand hue (`accent` is `ink`).
- Set secondary lines and every label in `inkSecondary`.
- The only bright colour is the musician's highlighter: one of eight pastels they choose (`marker-butter` to `marker-powder`), `marker` (butter) by default. Use it only as a fill with `ink` on it (11.3:1 on butter): the one primary button on a screen (MarkerButton), the swipe under page titles and the celebration glyph. Never set text or an outline in it; butter on paper is about 1.1:1.
- Danger is `danger`, a brick red; success is `success`, an olive green. Both lean warm.
- A piece is blue-grey (`pieceBadgeBg`), an exercise is sand (`exerciseBadgeBg`), with ink on both. Colour helps you spot the kind but is never the only signal.

## Cards

- Group content on white cards (`cardFill`) with a one point `hairline` edge, a 3 point corner (`radius-card`) and a faint lift (`shadow-card`). See Card.
- Give each entry in a screen's top-level list a card of its own, `spacing-cardCompact` (12) apart, as the Library's items and the Practice home's sessions do.
- Draw a list inside a detail card, such as a piece's linked exercises, sections or variations, or the items in a session being built, as rows in that card with a `divider` line (HairlineDivider) between them. A form's fields sit the same way.
- Inset content `spacing-card` (16) from the card's edge, or `spacing-cardCompact` (12) in a dense list. Space sections `spacing-section` (24) apart and controls `spacing-controlGap` (8).
- The small steps are named for their size: `spacing-xxSmall` (2) for a 2 point nudge, `spacing-xSmall` (4) for a label over its value, and `spacing-small` (6) for tags in a row.

## Controls

- Draw a stepper and a single button other than the one primary MarkerButton as ink on the card, so it recedes behind the content it changes.
- A stepper's minus and plus are ink glyphs inside thin circles, stroked one point in `divider`: about 32 points drawn, with a 44 point tap area. The value sits between them in DM Mono.
- Never put a white box, a filled panel or a sunken strip round a stepper's minus and plus, or round a single button.
- The catalogue's own looks stay as they are. Choosing one of a few options is SegmentedPills: a white track with a hairline edge, the chosen pill filled in ink. Giving a mark is ScoreSelector, with its ink filled boxes.
- A text field keeps its white FieldCard or FormField look, so it reads as something you can type into.
- Split rows with a hairline divider (HairlineDivider); never box them.
- A stepper reads aloud as one adjustable control.
- A tempo is a drag value (DragValue): the readout in Hanken Grotesk with tabular digits, as the metronome sets it, with a faint up and down glyph, dragged up or down in steps, typed on a tap. No rings.
- When last time's value is shown beside today's, a word tells them apart as well as `inkSecondary`.
- A small edit that belongs to the screen underneath, such as a key's tempo and note, opens in a BottomSheet rather than expanding a row in place.
- Rejected: white boxed buttons on a grey filled panel (#3004, T39 in [the design principles](../../docs/design-principles.md)).

## Type

Hanken Grotesk for every word; DM Mono only for a value that is a number alone, such as a clock, a duration or a count. Five sizes, and weight and colour carry the rest (T35):

| Size | Styles | For |
|---|---|---|
| 30 | `pageTitle` | The page title |
| 20 | `title`, `figureLarge` | A hero or the key picker, a number that leads its row |
| 17 | `cardTitle`, `body`, `bodyMedium` | Card titles, body, fields and values, emphasis |
| 15 | `label`, `secondary`, `figure`, `button`, `segment` | Labels, secondary lines, figures, buttons, pills |
| 13 | `small`, `smallMedium`, `badge` | Tabs, badges and chips |

Every label (a section's, a field's, a toggle's) is `label`: 15 medium in `inkSecondary`, sentence case, above what it names. Nothing is set in capitals.

## Writing

Plain British English in sentence case. A button starts with a verb, in two or three words: "Start session", "Save session". The first run says what the app is in one line: "A notebook for your practice."

## Components

**Before drawing a screen,** build it from the components below. If one almost fits, the change starts on its page in the repo; never draw a one-off copy. Each component's page gives its purpose, when to use it and when not, its variants, the tokens it is drawn with, its accessibility rules and its code, with pictures from the app's snapshot tests. These are native SwiftUI components: the pictures are the real thing. Five of them, ScreenScaffold, Card, LibraryItemCard, MarkerButton and TabBar, also have a web copy for sketching new screens (the sketch kit, below).

| Group | Component | What it is for |
|---|---|---|
| Layout | ScreenScaffold | Every top-level screen's chrome, safe areas and background |
| Navigation | TabBar | The four tabs along the bottom of the app |
| Navigation | ProfileBadge | The musician's instrument, the way into their profile |
| Surfaces | Card | The white surface every grouped block of content sits on |
| Surfaces | FieldCard | A form field with its label inside the card |
| Surfaces | FormField | A labelled text field in a form |
| Surfaces | BottomSheet | A short task over the current screen |
| Surfaces | ReflectionKeySheet | One key's tempo and note after an item, opened from its name |
| Surfaces | HairlineDivider | The 1 point line between rows |
| Actions | MarkerButton | The one call to action on a screen, in the musician's highlighter |
| Actions | SegmentedPills | Choosing one of two to five options in place |
| Actions | ScoreSelector | Giving an item, a play or a session its mark |
| Actions | ScoreStepper | Nudging a key's score from last time, with minus and plus |
| Actions | DragValue | Setting a tempo or other number by dragging it up or down |
| Actions | WeekStrip | Picking a day of the week on the Practice home |
| Actions | AddRowButton | The full-width row that adds something to a list |
| Actions | SwitchRow | A setting turned on or off, inside a card |
| Text and labels | SectionHeader | A section's title with an optional action |
| Text and labels | SectionTitle | The secondary-ink label above a section |
| Text and labels | FieldLabel | A field's label, in secondary ink |
| Text and labels | TagChip | A tag on an item, removable when editing |
| Text and labels | TypeBadge | Piece or exercise, at a glance |
| Feedback | GlobalBanner | An error or notice for the whole app |
| Feedback | FormErrorBanner | Why a form could not be saved |
| Feedback | MasteryDeltaToast | The celebration banner for the biggest rise in a mark |
| Feedback | PlaceholderContent | An empty screen or list, with what to do next |
| Progress | ScoreRing | A score from 1 to 10, or none yet |
| Progress | SegmentedProgress | Progress through a session's items |
| Progress | RecentSessions | An item's marks from its last few sessions, with the trend |
| Progress | MasteryDial | The average mark across the library, as a ring |
| Progress | ConsistencyBars | Minutes practised each week, as bars |
| Lists | LibraryItemCard | A piece or exercise in a list, with its mark |
| Lists | SessionCard | One past practice session, with its time, items and what was played |

The Library screen card shows how they sit together on a real screen.

Where a component still falls short of its page, the page says so in one line naming the GitHub issue that settles it.

## Sketching a screen

The sketch kit is a web copy of five components, `ScreenScaffold`, `Card`, `LibraryItemCard`, `MarkerButton` and `TabBar`, drawn from the tokens so a Claude Design canvas shows the real look of the app. It is a copy of the code, never the source (T38). A sketch that needs anything else draws it from the tokens and names the catalogue component it stands for, so the build knows what to reach for.

Where the kit follows a decision rather than the code (Jon, 15 October 2026):

- **Cards have the lift**, as T24 decided. The kit draws it by default (`lift={false}` turns it off); the app does not draw it until #2638 lands.
- **The tab bar is drawn as it looks on a real iPhone**, an opaque bar in `surfaceSunken`. The white bar in the iPhone snapshot is a test artefact (#2607). Its labels use `smallMedium`, the smallest type style, though the phone draws them a little smaller in the system font: the kit keeps to the theme's type.

### Phone frames

`ScreenScaffold` takes `device: "ios"` and then draws the whole phone: the status bar, the screen, and the tab bar when `tab` names one. The sizes are the device the snapshot tests use.

| | iPhone 13 |
|---|---|
| Screen | 390 by 844 points |
| Status bar | 47 |
| Navigation bar | 44 on a pushed screen, none at a tab's root |
| Tab bar | 49, then 34 for the home indicator |

```js
const { ScreenScaffold, LibraryItemCard, MarkerButton } = window.Intrada;

<ScreenScaffold device="ios" tab="library" title="Library" subtitle="8 pieces, 4 exercises" addLabel="Add item">
  <LibraryItemCard kind="piece" title="Clair de lune" subtitle="Debussy" score={7} />
</ScreenScaffold>
```

### Pushed screens

A screen opened from another one sits in its parent's tab. Pass `pushed` and the parent's `tab`: the tab bar stays, with the parent tab selected, and a back chevron sits above the title.

```js
<ScreenScaffold device="ios" tab="practice" pushed title="Session" />
```

### Consuming

How a Claude Design canvas uses the kit. The canvas type's own reference on design system components governs how files reach a canvas; this section says what the kit needs. The files are `project/components/tokens.css`, `bundle.css`, `bundle.js` and `index.d.ts`, and the fonts in `project/fonts/`.

1. **Make sure `window.React` and `window.ReactDOM` are React 18.** The kit reads `window.React.createElement`, so an ES module import is not enough. Only when the canvas does not already provide React 18, load React 18.3.1 and ReactDOM 18.3.1 as UMD scripts; never load a second copy.
2. **Link `tokens.css`, then `bundle.css`**, and load `bundle.js` after React. A canvas takes each as an uploaded file and links the address the upload returns. Upload the fonts in `project/fonts/` too and point the font faces in `tokens.css` at their uploaded addresses, or the text falls back to the system font.
3. **Take the components from `window.Intrada`.** Set the canvas page on `var(--paperTop)` in `var(--font-sans)`.

Text styles are CSS classes: `tokens.css` defines a class per type style, with general names such as `.body`, `.title`, `.label`, `.button` and `.small`, and they apply to the whole canvas page, not only the kit's components.

| Component | Props (`index.d.ts`) | Children |
|---|---|---|
| `ScreenScaffold` | `title`; optional `subtitle`, `device` (`"ios"`), `tab` (`"library"`, `"practice"`, `"routines"`, `"progress"`), `pushed`, `addLabel`, `actions`, `marker`, `noSwipe`, `reserveSubtitle` | The screen's content, under the header rule |
| `Card` | Optional `lift` (on unless `false`), `compact`, `style` | What the card holds |
| `LibraryItemCard` | `title`, `kind` (`"piece"` or `"exercise"`); optional `subtitle`, `keyName`, `tempo`, `priority`, `linkedExercises`, `keys`, `variations`, `tags`, `score` (0 to 10, or `null`), `showsMastery` | None |
| `MarkerButton` | `label`; optional `marker` | None |
| `TabBar` | Optional `device`, `selected` | None |

`marker` is one of `butter`, `coral`, `mint`, `sky`, `lavender`, `sage`, `peach` or `powder`; butter by default.

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

`TOKENS_CSS`, `BUNDLE_CSS` and `BUNDLE_JS` stand for the addresses the canvas's uploads return.

### Icons

The kit's icons come from vector drawables kept in the repo (`design/sketch-kit/drawables/`, moved there from the old Android app); each draws the same job as an SF Symbol, and `bundle.js` carries them. A symbol with no drawable is named in words on a sketch; SF Symbols themselves cannot be shipped as files. Inside the components the icons take the text colour.
