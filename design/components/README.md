# Component catalogue

Every component the screens are built from, with one page each: what it is
for, how it looks and behaves on iPhone and Android, and pictures from the
snapshot tests. Agents and people read the same pages; the Claude Design
system shows a copy, published from here and never edited there (#2620).

**Before building a screen,** reach for a component below. If one almost fits,
add a parameter to it. Never ship a one-off copy: hand-rolled copies are the
first source of drift between the apps.

**Changing a component** starts with its page: change the words, then the
SwiftUI version, then the Compose one, then record the snapshots on both
(`docs/design-workflow.md`). A new component starts as a new page from
[`_template.md`](_template.md) and a row here.

`scripts/check-components.sh` runs in `just check` and fails when a row has no
page, a page has no row, or a page names a file, symbol or snapshot that does
not exist. Rows without a link are still to be written.

| Component | What it is for |
|---|---|
| [Card](Card/README.md) | The white surface every grouped block of content sits on |
| [SegmentedPills](SegmentedPills/README.md) | Choosing one of two to five options in place |
| [MarkerButton](MarkerButton/README.md) | The one call to action on a screen, in the musician's highlighter |
| TagChip | A tag on an item, removable when editing |
| TypeBadge | Piece or exercise, at a glance |
| ScoreRing | A score from 1 to 5, or none yet |
| BottomSheet | A short task over the current screen |
| GlobalBanner | An error or notice for the whole app |
| FormErrorBanner | Why a form could not be saved |
| PlaceholderContent | An empty screen or list, with what to do next |
| ScreenScaffold | Every top-level screen's chrome, safe areas and background |
| SectionHeader | A section's title with an optional action |
| HairlineDivider | The one-pixel line between rows |
| SegmentedProgress | Progress through a session's items |
| SectionTitle | The secondary-ink label above a section |
| FieldCard | A form field with its label inside the card |
| FieldLabel | A field's label, in secondary ink |
