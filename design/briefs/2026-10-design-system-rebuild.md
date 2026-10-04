# Rebuild the design system page to match the app (#1678)

*2026-10-02. Paste into a Claude Design session working on
`Intrada Design System.dc.html` (`design/intrada-design-system.dc.html` in the
repo).*

## Goal

Rebuild the design system page so every token, component and glyph on it is
what the iOS app draws today. The page is a picture of
`ios/Intrada/DesignSystem/Theme.swift`, not a proposal.

**Values come only from this brief.** Do not keep a value because the current
page has it, and do not invent one to fill a gap: if a section needs a value
this brief does not give, leave the section out and say so.

## Colour

Paper and surfaces:

| Name | Hex | Role |
|---|---|---|
| paperTop | #F7F4EF | Page ground everywhere |
| cardFill | #FFFFFF | Cards and inset surfaces |
| surfaceSunken | #F3EFE8 | Sunken chips and recessed rows inside a card |
| tabBarFill | #F3EFE8 | Tab bar |
| hairline | #ECE6DA | Card edge, 1pt divider |
| divider | #DDD7CA | Stronger divider |
| slotOutline | #DDD7CA | Empty rep slot ring, missed button border |
| addDashOutline | #C9BFB0 | Dashed outline on "+ Add" rows |
| viewerBackdrop | #1A1917 | Full-screen photo viewer ground |
| sheetScrim | black at 20% | Dimmed ground behind a sheet |

Ink:

| Name | Hex | Role |
|---|---|---|
| ink | #2A2725 | Text, and every interactive colour on paper |
| inkSecondary | #6E6A66 | Metadata and secondary text |
| inkFaintIcon | #8F8070 | A dimmed glyph with no text of its own; never text |
| inkFainter | #C2B8AA | Placeholder and unavailable glyphs |
| accent | = ink | No brand hue: the interactive colour is ink |
| onAccent | #FFFFFF | Text on an ink fill |
| onMarker | = ink | Text on a highlighter fill |

Semantic and type:

| Name | Hex | Role |
|---|---|---|
| danger | #9C4A3A | Destructive actions, refusals |
| dangerWash | danger at 10% | Wash on a field a banner points at |
| dangerBanner | danger at 12% | Error banner ground |
| dangerEdge | danger at 25% | Edge on a faulted field |
| success | #4C6B3F | Clean reps, gains |
| repCleanBg / repCleanBorder | #EDF1EA / #D3DDCC | Clean rep chip |
| repMissedFg / repMissedBg | #756A5C / #F3EFE8 | Missed rep: taupe, never red |
| masteryTrack, dialTrack, timerTrack, consistencyTrack | #EDE8DC | Empty track under rings and bars |
| masteryFill | = ink | Mastery is monochrome; never recolour by level |
| pieceBadgeBg | #CBD6E0 | Piece tag and card bar (blue-grey), ink text |
| exerciseBadgeBg | #DCD3C0 | Exercise tag and card bar (sand), ink text |
| celebrationBg / celebrationInk | = ink / #F7F4EF | Dark celebration toast |
| playerBgTop / Mid / Bottom | #FBFAF7 / #F7F4EF / #EFEAE1 | Player radial wash, centre 50% 22%, radius 440 |
| shadow / buttonShadow | ink at 5% / ink at 6% | Card and button lift |

## The eight highlighters

The musician picks one; it is the only bright colour in the app. Butter is the
default. It fills primary buttons, the hero's start button, the swipe under
page titles and the celebration glyph, always with ink on it. Interactive
colour on paper stays ink: a highlighter is never text or an outline.

The Practice hero card is a gradient from `ink` (#2A2725, top trailing) to the
highlighter's own deep stop (bottom leading).

| Highlighter | Fill | Hero bottom stop | Ink on fill |
|---|---|---|---|
| Butter (default) | #FFE9A3 | #4F3B28 | 12.32:1 |
| Coral | #FFB4A2 | #4C2C24 | 8.69:1 |
| Mint | #B9EBD3 | #244C39 | 11.22:1 |
| Sky | #A9D8FF | #243A4C | 9.86:1 |
| Lavender | #D6CCF5 | #2E244C | 9.76:1 |
| Sage | #CFDDB0 | #3D4923 | 10.33:1 |
| Peach | #FFD7B0 | #4C3824 | 11.02:1 |
| Powder | #C7DCE8 | #243D4C | 10.48:1 |

The title swipe is the highlighter as a band behind the lower half of the
title: clear to 50%, soft at 53%, full from 58% to 89%, soft at 93%, clear from
96%, with 3pt corners. Soft is 60% opacity.

## Contrast on paper and cards

Measured with the WCAG formula from the hexes above.

| Colour | On paper #F7F4EF | On card #FFFFFF | Floor it must clear |
|---|---|---|---|
| ink | 13.52:1 | 14.84:1 | 4.5:1 text |
| inkSecondary | 4.89:1 | 5.36:1 | 4.5:1 text |
| inkFaintIcon | 3.49:1 | 3.83:1 | 3:1 non-text glyph |

## Type

Hanken Grotesk for titles and body, DM Mono for figures. Five sizes, 30, 20,
17, 15 and 13; weight and colour carry the hierarchy inside a size (T35 in
`docs/design-principles.md`, #1881). Every style scales with Dynamic Type from
its anchor.

| Style | Face | Size (pt) | Weight | Anchor |
|---|---|---|---|---|
| pageTitle | Hanken Grotesk | 30 | SemiBold | Large Title |
| title | Hanken Grotesk | 20 | SemiBold | Title 3 |
| cardTitle | Hanken Grotesk | 17 | SemiBold | Headline |
| body | Hanken Grotesk | 17 | Regular | Body |
| bodyMedium | Hanken Grotesk | 17 | Medium | Body |
| label | Hanken Grotesk | 15, inkSecondary, sentence case | Medium | Subheadline |
| secondary | Hanken Grotesk | 15, tabular digits | Regular | Subheadline |
| figure | DM Mono | 15 | Regular | Subheadline |
| button | Hanken Grotesk | 15 | Bold | Subheadline |
| segment | Hanken Grotesk | 15 | Medium | Subheadline |
| small | Hanken Grotesk | 13 | Regular | Caption |
| smallMedium | Hanken Grotesk | 13 | Medium | Caption |
| badge | Hanken Grotesk | 13 | SemiBold | Caption |
| timer | Hanken Grotesk | 56, tabular digits | SemiBold | Large Title |
| scoreNumeral | Hanken Grotesk | set per use | SemiBold | Title 3 |
| chart | system monospaced | Footnote | Regular | Footnote |
| chartEditor | system monospaced | Body | Regular | Body |

Nothing is set in capitals. Every label, a section's title, a field's label
(Key, Tempo) or a toggle's title, is `label` in `inkSecondary`, in sentence
case, at the top of what it names; `SectionTitle` and `FieldLabel` look the
same, and a field's label sits inside its card above the value (`FieldCard`).
Values, and anything read or set, are 17 in ink. Only the dark Practice and Up
next cards keep their labels light on dark. Mono is
for a figure set alone (a clock, a duration, a count); a line that mixes words
and numbers is `secondary`. The timer and the score numerals are the only
styles sized by the dial or readout they sit in.

## Spacing, radius, lift, opacity

| Spacing | pt | | Radius | pt |
|---|---|---|---|---|
| controlGap | 8 | | card, control, badge, panel, hero | 3 |
| cardCompact | 12 | | pill | fully round |
| card | 16 | | | |
| section | 24 | | | |

Cards are white with a 1pt `hairline` edge, 3pt corners and a faint lift. They
are not flat.

| Shadow | Colour | Blur radius | y offset |
|---|---|---|---|
| card | ink at 5% | 1 | 1 |
| button | ink at 6% | 1 | 1 |
| lifted | ink at 5% | 6 | 3 |
| glow | ink at 40% | 6 | 4 |
| transport | ink at 18% | 14 | 6 |
| heroButton | black at 25% | 16 | 8 |
| hero | black at 18% | 20 | 10 |

| Opacity | Value | Job |
|---|---|---|
| wash | 0.12 | Faint tint over a surface |
| dimmed | 0.5 | Disabled or superseded |
| soft | 0.6 | Softened fill that still reads as its colour |
| secondary | 0.75 | Secondary text on a dark or photo ground |
| strong | 0.8 | Ground or text that must hold contrast over anything |

## Icon sizes

SF Symbols, scaling with Dynamic Type up to a cap.

| Size | pt | Cap | | Instrument glyph | pt |
|---|---|---|---|---|---|
| badge | 9 | 13 | | bar | 36 |
| caption | 11 | 16 | | tile | 56 |
| inline | 15 | 18 | | hero | 88 |
| control | 20 | 26 | | | |
| large | 28 | 36 | | | |
| transport | 32 | 40 | | | |
| hero | 38 | 56 | | | |

## Motion

| Token | Value |
|---|---|
| standard | spring, response 0.35, damping 0.85 |
| snappy | spring, response 0.28, damping 0.9 |
| gentle | spring, response 0.45, damping 0.82 |
| fadeUp | opacity 0 to 1, rise 12pt, 500ms ease-out, +60ms per item, once |
| barGrow | scale up from the baseline, 600ms, cubic-bezier(0.2, 0.8, 0.3, 1), +60ms per bar |
| countUp | 1.5s ease-out, the mastery dial number and ring |
| pop | spring from 0.82 to 1, response 0.35, damping 0.62 |
| press | dip to 0.94 on press, snappy spring back |

Reduce Motion: every reveal renders its final state with no animation, and the
press dip does not happen. There is no fade.

## Components in the app

Show these, by these names, and nothing else as a component.

1. **Foundations** (`ios/Intrada/DesignSystem`): `ScreenScaffold`,
   `.cardSurface()`, `.cardShadow()`, `HairlineDivider`, `TagChip`,
   `GlobalBanner` (danger with `exclamationmark.triangle.fill`, notice with
   `info.circle`), `FormErrorBanner`, `PlaceholderContent`, `ProfileBadge`,
   `InstrumentGlyph`, `.markerSwipe()`, `.scrimCapsule()`, `FieldMark`
   (`doc.viewfinder`), `FaultMark`, `.scrollEdgeShadow()`.
2. **Reusable views** (`ios/Intrada/Views/Components`): `SectionHeader` and
   `SectionTitle`, `FieldCard` and `FieldLabel`, `TypeBadge`, `LibraryItemCard`, `SegmentedPills`,
   `SegmentedProgress`, `BottomSheet`, `ScoreRing`, `ScoreSelector`,
   `MasteryDial`, `MasteryHeroCard`, `ConsistencyBars`, `WeekStrip`,
   `RepCounter` ("Got it" `checkmark`, missed `xmark`), `TransportButton`
   (`play.fill`, `forward.end`), `UpNextHero`, `SessionCard`, `AddRowButton`
   (`plus`), `TempoStepper`, `ClickControl` (`metronome`), `FormField`,
   `FormSectionRow`, `PhotoCard`.
3. **Tab bar**: Library `books.vertical`, Practice `timer`, Routines
   `music.note.list`, Progress `chart.line.uptrend.xyaxis`.
4. **Item kinds**: Piece `music.note` on blue-grey, Exercise `dumbbell.fill` on
   sand. The type badge is a pill with glyph and text, never colour alone.

## Remove from the page

- Indigo #4C3FA6 and every brand gradient built on it.
- The gold exercise accent.
- Inter and Source Serif.
- Radii of 12 to 16px.
- Any claim that cards are flat.
- The 640px breakpoint.
- The 150ms Reduce Motion fade.
- MasteryMeter, WeekPicker, SessionBlock and RelatedPicker.
- The Coach primitives section.
- The metronome as a tab icon.
- The repeat glyph.
- clickActiveBg.

## Done looks like

- [ ] Every hex on the page appears in this brief, and every colour in this
      brief appears on the page.
- [ ] All eight highlighters are shown with their hero gradients, butter first.
- [ ] Type specimens use Hanken Grotesk and DM Mono only, at the sizes above.
- [ ] Every card has 3pt corners, a hairline edge and the card lift.
- [ ] The contrast table matches the figures above.
- [ ] Components carry their code names and real SF Symbol names.
- [ ] Nothing from the remove list is left, searched for by name and hex.
- [ ] The Reduce Motion note says final state, no animation.
- [ ] The shareable `design/intrada-design-system.html` is re-exported from the
      new page.
