# Design audit, 2026-10 (#2599)

Both native apps, every screen and sheet, read from the committed snapshots
against `Theme.swift`, the primitive list in `.claude/rules/ios-ui.md`, the
Android rules in `.claude/rules/android-quality.md` and the T-decisions in
`docs/design-principles.md`. Three read-only reviews ran in the
`design-audit-2599` worktree at e3cd2990; nothing was built or run, so I4
(the iOS 26 tab bar) needs a device before it counts.

The review page, with every finding beside its snapshots, is
https://claude.ai/artifact/5djCUQ6eUx8EvAPirGQDXY.

## Summary

- **iPhone:** 40 findings: 3 serious, 1 to check on a device, 21 visible, 15
  code only. 22 surfaces have no snapshot; 3 are real gaps (photo viewer,
  draft exercise sheet, chord chart editor), the rest are system alerts and
  menus.
- **Android:** 30 findings: 3 serious, 23 visible, 4 code only. 18 surfaces
  have no snapshot: no test opens a dialog or menu.
- **Android behaviour:** 17 areas reviewed; the draft principle is T36 in
  `docs/design-principles.md`, proposed.
- **Verified by the lead in the code:** A1, A2, A4, I1, I2, I36, the
  reflection screen's back handler (deliberate, with a comment) and the
  missing ripple.

## What stands out

1. Android does not use the musician's highlighter: primary buttons are black,
   there is no swipe under page titles, and the chosen colour never reaches
   the toast or the metronome (A4 to A6).
2. Android never shows a notice (A1), and an off switch is a white dot on
   paper (A2).
3. iPhone cards have lost their lift: the shared card style draws a hairline
   and no shadow in 55 places, with the shadow added in only 4 (I36).
4. On iPhone the highlighter fills primary buttons and also half the selected
   choices (I9), and a ticked choice has five looks (I10).
5. Smaller but seen: the gap in "D ♭ major" (I20), the tab bar drawing white
   in snapshots (I4), Discard on the recovery card with no confirm (I2).

## iOS design audit, 2026-10 (#2599)

Read-only, against main at e3cd2990 in the `design-audit-2599` worktree. Nothing built or run. All 185 committed snapshot images were opened and looked at; every finding was checked against the code at the line given. Where a finding rests on reading code rather than a picture, it says so. Snapshot paths are relative to `ios/IntradaTests/__Snapshots__/` unless they start with `ios/`. The snapshot host pins `en_US`, so "Jun 24" and "9:02 AM" in pictures are V5 working as designed, not findings.

**Last month's audit (docs/audit-2026-09.md):** its design-side findings are closed. 8 (Repetitions) was reversed by decision (design-principles.md:748-750, #1938 closed). 11 (faint ink on text) is gone: `inkFaint` no longer exists in Theme.swift and #1941 is closed. 23 (raw `.font(.system(size:))`) is gone: none remain in `ios/Intrada/Views`, #1950 closed. 9 (#1939, closed) is closed for keys, but its tempo half survives as I8 below.

### (a) Inventory

Kinds: screen, cover (full-screen), sheet, dialog (alert), menu (menu, long-press or swipe), banner (banner, card or inline offer that interrupts a screen).

| Surface | Kind | Snapshot(s) |
|---|---|---|
| RootView (tab shell) | screen | RootShellSnapshotTests/testRootShell.1.png, testRootShellHalted.1.png, testRootShellWithNotice.1.png |
| LibraryScreen | screen | LibrarySnapshotTests/testLibraryScreen.1.png, testLibraryScreenPopulated, testLibraryScreenFiltered, testLibraryScreenMastery, testLibraryScreenPriorities, testLibraryScreenSearching, testLibraryScreenSearchMatchesNothing, testLibraryScreenSearchRevealedEmpty, testLibraryScreenEmptyWithSearch (all .1.png) |
| LibrarySplitView (iPad) | screen | LibrarySnapshotTests/testLibrarySplitViewEmptyDetail.1.png, testLibrarySplitViewEmptyLibrary.1.png, testLibrarySplitViewWithSelection.1.png |
| LibraryDetailScreen (piece and exercise) | screen | LibraryDetailSnapshotTests/testLibraryDetailScreen, testLibraryDetailChordChartCard, testPieceDetailWithSections, testPieceDetailLinkedEmpty, testPieceDetailLinkedPopulated, testPieceDetailLinkedEditing, testExerciseDetailWithVariations, testExerciseDetailWith12Variations, testExerciseDetailWithKeys, testExerciseDetailWithLongVariationName, testExerciseDetailWithNamedVariations, testExerciseDetailVariationsEmptyState, testExerciseDetailUsedIn, testExerciseDetailUsedInLinkedOnly, testSectionsReordering, testRelatedExercisesWithSectionLinks, testRelatedExercisesSectionLinksEditing, testUsedInCardOnItsOwn, testUsedInCardRowStates, testUsedInWithSections, testTempoTrend, testRecentSessions, testRecentSessionsDeclining, testPhotoCardStates (all .1.png) |
| PracticeScreen | screen | PracticeSnapshotTests/testPracticeScreen, testPracticeScreenPopulated, testPracticeScreenGreeting, testPracticeScreenQuietDay, testPracticeScreenDayStillToCome, testPracticeScreenPriorities, testPracticeScreenSuggestion, testPracticeScreenSuggestionPriorities, testPracticeScreenSuggestionDismissed, testPracticeScreenSuggestionDismissedPriorities, testUpNextHeroCoral, testUpNextHeroFilledPlan, testUpNextHeroNeverMarked, testSessionCardsPlayedSummary (all .1.png); FirstRunSnapshotTests/testPracticeScreenStartHere.1.png |
| PracticeSplitView (iPad) | screen | PracticeSplitSnapshotTests/testPracticeSplitViewEmptyDetail.1.png, testPracticeSplitViewWithSelection.1.png |
| PracticeSessionDetailScreen | screen | PracticeSnapshotTests/testPracticeSessionDetail, testPracticeSessionDetailAccessibilitySize, testPracticeSessionDetailEndedEarly, testPracticeSessionDetailWithOneVariation, testPracticeSessionDetailWithVariations (all .1.png) |
| SessionBuilderScreen | screen | SessionBuilderSnapshotTests/testSessionBuilderEmpty, testSessionBuilderEmptyWithLength, testSessionBuilderPopulated, testSessionBuilderPlanned, testSessionBuilderGrouped, testSessionBuilderGroupedEditing (all .1.png) |
| AnalyticsScreen (Progress) | screen | ProgressSnapshotTests/testAnalyticsScreen, testProgressScreenPopulated, testProgressScreenNothingThisWeek, testConsistencyBars, testMasteryDial, testMasteryDialTakesTheTopMarkFromTheCore, testMasteryDeltaRows, testMasteryDeltaRowAfterAFall (all .1.png) |
| RoutinesScreen | screen | ProgressSnapshotTests/testRoutinesScreen.1.png |
| ProfileScreen | screen | ProfileSnapshotTests/testProfileScreen, testProfileScreenEmpty, testProfileScreenFull, testHighlighterColours, testInstrumentIcons (all .1.png) |
| FocusPlayerScreen (via PlayerHost) | cover | FocusPlayerSnapshotTests/testFocusPlayerWithReps, testFocusPlayerWithTarget, testFocusPlayerWithVariations, testFocusPlayerLongSession, testFocusPlayerLastTimeOffer, testFocusPlayerSectionTimeUp, testFocusPlayerAwayOffer, testFocusPlayerHoldsTheItemTimerWhileTheSheetIsOpen, testOrientationBandLongSession, testRepCounter, testRepCounterAccessibilitySize, testClickControlStates, testClickControlDraggingStates, testClickBarLineStates, testGroupedPillsWrap (all .1.png) |
| SessionSummaryScreen | cover | SessionSummarySnapshotTests/testSessionSummaryCompleted, testSessionSummaryEndedEarly, testSessionSummaryWithVariations, testScoreSelectorPills (all .1.png) |
| FirstRunScreen (welcome, profile step, first piece) | cover | FirstRunSnapshotTests/testWelcome, testWelcomeOnIPad, testProfileStep, testFirstPieceStep (all .1.png) |
| LaunchSplash, ReturningSplash | cover | FirstRunSnapshotTests/testLaunchSplash.1.png, testReturningSplash.1.png |
| PageCameraScreen | cover | LibraryFormSnapshotTests/testPageCameraConfirm.1.png, testPageCameraStates.1.png |
| PhotoViewer | cover | UNCOVERED |
| LibraryAddScreen (ItemFormScaffold) | sheet | LibraryFormSnapshotTests/testLibraryAddScreen, testLibraryAddScreenExercise, testLibraryAddScreenStaged, testLibraryAddScreenWithError, testLibraryAddScreenReadFromAPhoto, testLibraryAddScreenAfterEditingAReadField, testLibraryAddScreenMarksTheFieldAtFault, testLibraryAddScreenMarksTheStagedRowInContext, testLibraryAddScreenMarksTheVariations, testScanPageEntryStates, testStagedChartCardAtFault, testDraftItemRows, testDraftItemRowAtFault, testKeyPickerCollapsed, testKeyPickerExpandedEmpty, testKeyPickerExpandedEnharmonic, testAutocompleteField, testTagChipInput (all .1.png) |
| LibraryEditScreen (ItemFormScaffold) | sheet | LibraryFormSnapshotTests/testLibraryEditScreen, testLibraryEditScreenExercise, testLibraryEditScreenExerciseWithVariations, testLibraryEditScreenWithError (all .1.png) |
| DraftExerciseSheet | sheet | UNCOVERED |
| ChordChartEditSheet | sheet | UNCOVERED |
| LinkedItemPickerSheet | sheet | LibraryFormSnapshotTests/testLinkedExercisePicker.1.png, testLinkedExercisePickerWithDraft.1.png, testLinkedPiecePicker.1.png |
| AddRelatedExerciseSheet | sheet | LibraryDetailSnapshotTests/testAddRelatedExerciseSheet.1.png, testAddRelatedExerciseSheetAdded.1.png |
| KeysSheet | sheet | LibraryDetailSnapshotTests/testKeysSheetAllMajor.1.png, testKeysSheetMixed.1.png |
| VariationsSheet | sheet | LibraryDetailSnapshotTests/testVariationsSheet.1.png, testVariationsSheetFindingOrAdding.1.png |
| LinkSectionsSheet | sheet | LibraryDetailSnapshotTests/testLinkSectionsSheet.1.png |
| SectionSheet | sheet | LibraryDetailSnapshotTests/testSectionSheetEditingATrickySpot.1.png, testSectionSheetRefused.1.png |
| ScaffoldPreviewSheet | sheet | LibraryDetailSnapshotTests/testScaffoldPreviewSheet.1.png |
| TagFilterSheet | sheet | LibrarySnapshotTests/testTagFilterSheet.1.png, testTagFilterSheetEmpty.1.png |
| AddToSessionSheet | sheet | SessionBuilderSnapshotTests/testAddToSessionSheet, testAddToSessionSheetRecentlyPractised, testAddToSessionSheetRecentlyPractisedHiddenWhileFiltered, testAddToSessionSheetLargestStandardSizeLongKeyAndTempo (all .1.png) |
| EntrySettingsSheet | sheet | SessionBuilderSnapshotTests/testEntrySettingsSheetEmpty, testEntrySettingsSheetPopulated, testEntrySettingsSheetKeyRow, testEntrySettingsSheetOffersDrillsOneTicked, testEntrySettingsSheetSplitWithFocus, testEntrySettingsSheetSuggestsAFocus (all .1.png) |
| EntryKeyList (pushed inside entry settings) | sheet | SessionBuilderSnapshotTests/testEntryKeyList.1.png |
| ClickSheet | sheet | FocusPlayerSnapshotTests/testClickSheetIrregularMetre.1.png, testClickSheetLongBar.1.png |
| VariationPickerSheet | sheet | FocusPlayerSnapshotTests/testVariationPickerSheet.1.png |
| TroubleSpotSheet | sheet | FocusPlayerSnapshotTests/testTroubleSpotSheet.1.png |
| ReflectionSheet | sheet | SessionSummarySnapshotTests/testReflectionSheet, testReflectionSheetChangingWhatWasPlayed, testReflectionSheetWithADroppedPlay, testReflectionSheetWithARefusedSave, testReflectionSheetWithFinishAnswers, testReflectionSheetWithTempoPerRow, testReflectionSheetWithTempoPerRowAccessibilitySize, testReflectionSheetWithTempoTarget, testReflectionSheetWithTheAimReadFromThePlays, testReflectionSheetWithThreeVariations, testReflectionSheetWithThreeVariationsAccessibilitySize (all .1.png) |
| ProfileEditSheet | sheet | ProfileSnapshotTests/testProfileEditSheet.1.png, testProfileEditSheetWithError.1.png |
| InstrumentIconPicker | sheet | ProfileSnapshotTests/testInstrumentIconPicker.1.png |
| FeedbackSheet | sheet | ProfileSnapshotTests/testFeedbackSheet.1.png, testFeedbackSheetFromAShake.1.png |
| "Discard this session?" (SessionSummaryScreen.swift:52) | dialog | UNCOVERED |
| "Discard session plan?" (SessionBuilderScreen.swift:160) | dialog | UNCOVERED |
| "Delete {title}?" (LibraryDetailScreen.swift:94) | dialog | UNCOVERED |
| "Remove {section}?" (SectionSheet.swift:59) | dialog | UNCOVERED |
| "Rename {variation}" text alert (VariationsSheet.swift:41) | dialog | UNCOVERED |
| "Remove this photo?" (PhotoCard.swift:52) | dialog | UNCOVERED |
| Speech problem alert (SpeechInput.swift:329) | dialog | UNCOVERED |
| "Remove {variation}?" (VariationRowsSection.swift:35) | dialog | UNCOVERED |
| Player options: Skip, End session early (FocusPlayerScreen.swift:185) | menu | UNCOVERED |
| Builder block menu: Just the piece, Ungroup, Remove block (SessionBuilderScreen.swift:537) | menu | UNCOVERED |
| Builder row swipe: Remove (SessionBuilderScreen.swift:657) | menu | UNCOVERED |
| Library kind filter (LibraryFilterMenu.swift:9) | menu | UNCOVERED (closed state only, in Library pictures) |
| Library sort (LibrarySortMenu.swift:64) | menu | UNCOVERED (closed state only) |
| Library row long-press: priorities (LibraryScreen.swift:100) | menu | UNCOVERED |
| Variation long-press: Rename (VariationsSheet.swift:123) | menu | UNCOVERED |
| Scan source menu (ScanPageEntry.swift:128) | menu | UNCOVERED |
| Photo card menu (PhotoCard.swift:66) | menu | UNCOVERED |
| Section split menu (EntrySegmentsCard.swift:64) | menu | UNCOVERED |
| Entry variation menu (EntrySettingsSheet.swift:169) | menu | UNCOVERED |
| GlobalBanner via AppBannerStack (danger, notice, halted, storage) | banner | RootShellSnapshotTests/testGlobalBanner.1.png, testRootShellHalted.1.png, testRootShellWithNotice.1.png |
| FormErrorBanner | banner | LibraryFormSnapshotTests/testLibraryAddScreenWithError.1.png, testLibraryEditScreenWithError.1.png; ProfileSnapshotTests/testProfileEditSheetWithError.1.png; LibraryDetailSnapshotTests/testSectionSheetRefused.1.png; SessionSummarySnapshotTests/testReflectionSheetWithARefusedSave.1.png |
| RecoveryPromptCard | banner | PracticeSnapshotTests/testRecoveryPromptCard.1.png |
| UpNextHero | banner | PracticeSnapshotTests/testUpNextHero*.1.png (3) |
| StartHereCard | banner | FirstRunSnapshotTests/testPracticeScreenStartHere.1.png |
| AwayOfferBanner | banner | FocusPlayerSnapshotTests/testFocusPlayerAwayOffer.1.png |
| Move-on offer (SegmentClockRow) | banner | FocusPlayerSnapshotTests/testFocusPlayerSectionTimeUp.1.png |
| Last-time OfferChip | banner | FocusPlayerSnapshotTests/testFocusPlayerLastTimeOffer.1.png; SessionBuilderSnapshotTests/testSessionBuilderPlanned.1.png |
| MasteryDeltaToast | banner | ProgressSnapshotTests/testMasteryDeltaRows.1.png, testProgressScreenPopulated.1.png |

**UNCOVERED: 22 surfaces** (3 sheets or covers, 8 dialogs, 11 menus or gestures). The dialogs and menus are system controls that `ios-quality.md:20-22` puts outside the gate; the three real gaps against "Every screen ships with a snapshot test" (`ios-quality.md:8`) are PhotoViewer, DraftExerciseSheet and ChordChartEditSheet.

Not surfaces, so not in the table: FeedbackPresenter, PhotoCapture+Sources, ReflectionHandoff, ItemFormModel, FlowLayout, DragReorder, PlayerHost, ListDetailSplit (host views), and the row and field components, which appear inside the snapshots above (ComponentSnapshotTests and LibrarySnapshotTests/testTypeBadges, testLibraryItemCards cover the smallest ones).

### (b) Findings

Counts: 3 high, plus 1 high only if a device confirms it (I4); 21 medium; 15 low. IDs I35 to I40 were added after a second pass and sit in their severity sections out of number order.

#### High

**I1. A chord-chart suggestion already added is faded to 60%, taking its words below the contrast floor**
- Surface: ScaffoldPreviewSheet. Snapshot: LibraryDetailSnapshotTests/testScaffoldPreviewSheet.1.png.
- Rule: T31, "Not played is said by a colour step, never a fade ... the words stay readable and only the glyph dims" (docs/design-principles.md:1076-1081).
- On screen: the "Learn the melody · Already added" row, title, reason line and badge alike, is pale grey on white; the secondary line is close to unreadable.
- Evidence: ScaffoldPreviewSheet.swift:155 (`.opacity(selectable ? 1 : IntradaOpacity.soft)` on the whole row).

**I2. Discard on the recovery card throws away an interrupted session in one tap, with no confirm**
- Surface: RecoveryPromptCard on Practice. Snapshot: PracticeSnapshotTests/testRecoveryPromptCard.1.png.
- Rule: "Reversible by default; confirm only the irreversible ... Reserve confirmation dialogs for genuinely destructive, non-recoverable actions" (docs/design-principles.md:103-105); T6 (:176-182). The summary's own Discard does ask ("Discard this session?", SessionSummaryScreen.swift:52-57), so one action has two patterns.
- On screen: "Discard" sits a thumb from "Resume"; tapping it clears the saved session with no question and no undo.
- Evidence: RecoveryPromptCard.swift:41-45; Store.swift:235-238 (`discardSessionInProgress` clears the slot outright). No logged decision on the recovery card's discard was found (grep of docs/design-principles.md for "recover" and "Pick up where").

**I3. Small controls without a 44pt target: the priorities star and the related-exercise move and remove buttons**
- Surface: Library, Add to session, Link a piece / Add exercises, piece detail (editing related exercises). Snapshots: LibrarySnapshotTests/testLibraryScreenPopulated.1.png, SessionBuilderSnapshotTests/testAddToSessionSheet.1.png, LibraryFormSnapshotTests/testLinkedExercisePickerWithDraft.1.png, LibraryDetailSnapshotTests/testRelatedExercisesSectionLinksEditing.1.png.
- Rule: the 44pt target the app holds itself to elsewhere (T25, "36pt in the same 44pt target as the Library's add button", docs/design-principles.md:938; RepCounter's `undoTarget = 44`) and Apple's HIG minimum. Also "Never ship a parallel one-off" (.claude/rules/ios-ui.md:49-51): the star pill is hand-copied twice.
- On screen: the star capsule is about 25 by 30pt; the up, down and minus glyphs on a related exercise are bare 15pt symbols with no padding.
- Evidence: BrowseControlsBar.swift:111-117 and LinkedItemPickerSheet.swift:140-145 (13pt glyph, 6/10 padding, no frame); RelatedExercisesCard.swift:300-325 (no `.frame` on the three buttons, compare VariationRowsSection.swift:101-104 which frames the same minus at 44).

**I4. The tab bar draws white on paper in every committed picture of it, and so does the back chevron (high only if a device confirms it)**
- Surface: RootView tab bar; the navigation back button on pushed screens. Snapshots: RootShellSnapshotTests/testRootShell.1.png, testRootShellHalted.1.png, testRootShellWithNotice.1.png; LibraryDetailSnapshotTests/testLibraryDetailScreen.1.png and every other pushed-screen picture.
- Rule: text contrast (WCAG 1.4.3), and "Every colour ... traces to IntradaColor" (.claude/rules/ios-ui.md:37-38).
- On screen: Library, Practice, Routines and Progress labels and glyphs are near-invisible white; the "<" at top left of the detail screens is barely there.
- Evidence: RootView.swift:150-151 says "iOS 26's glass tab bar styles itself and ignores these item colours". Severity is high only if a device shows the same; I could not run one. If it is a renderer artefact, the finding becomes: the only snapshot of the tab bar cannot catch a tab-bar regression.

#### Medium

**I5. The add form's title reads "New Piece" and "New Exercise", in title case**
- Surface: LibraryAddScreen. Snapshots: LibraryFormSnapshotTests/testLibraryAddScreen.1.png and the other LibraryAddScreen pictures.
- Rule: "Sentence case everywhere" (docs/tone-of-voice.md:123-125).
- Evidence: LibraryAddScreen.swift:36 (`"New \(form.kind.label)"`, where `label` is "Piece", ItemKindStyle.swift:35-38). DraftExerciseSheet.swift:40 gets it right ("New exercise").

**I6. "Skip rating" under the reflection sheet's Save**
- Surface: ReflectionSheet. Snapshot: SessionSummarySnapshotTests/testReflectionSheetWithThreeVariations.1.png.
- Rule: V4, the screen says Mark; `Rating` was "considered and rejected" as "the data model's word" (docs/tone-of-voice.md:342-357); rule 2 table (:78).
- Evidence: ReflectionSheet.swift:263.

**I7. The summary headline says "ended early"**
- Surface: SessionSummaryScreen; also the history detail and the session card. Snapshots: SessionSummarySnapshotTests/testSessionSummaryEndedEarly.1.png; PracticeSnapshotTests/testPracticeSessionDetailEndedEarly.1.png, testSessionCardsPlayedSummary.1.png.
- Rule: V1, "`Nice work.` goes, and so does its partner `Ended early.`; the summary leads with the facts it actually has" (docs/tone-of-voice.md:303-319).
- On screen: "2 of 3 · ended early" under the time.
- Evidence: SessionSummaryScreen.swift:76; SessionCard.swift:26. V1 names the summary; whether the history keeps it is Jon's ruling.

**I8. Tempo is written "66 bpm" on the history and summary rows, and "♩ = 66" everywhere else**
- Surface: PracticeSessionDetailScreen, SessionSummaryScreen. Snapshots: PracticeSnapshotTests/testPracticeSessionDetail.1.png, testPracticeSessionDetailWithVariations.1.png.
- Rule: rule 2, "BPM, quarter note → `♩ = 132`, crotchet" and "Music notation belongs on screen where a musician would write it" (docs/tone-of-voice.md:80, :84-85). Last month's #1939 (closed) fixed the keys half.
- Evidence: crates/intrada-core/src/view/session.rs:239 (`format!("{tempo} bpm")`, read through `metaParts` at PracticeSessionDetailScreen.swift:175-192); SessionSummaryScreen.swift:218 builds the same string in Swift.

**I9. The highlighter fills "selected" on half the choice controls; the other half select in ink**
- Surface: ReflectionSheet (Aim met, How it felt, From your note), EntrySettingsSheet focus chips, instrument icon picker, running metronome readout, Start here ticks. Snapshots: SessionSummarySnapshotTests/testReflectionSheetWithFinishAnswers.1.png, SessionBuilderSnapshotTests/testEntrySettingsSheetSuggestsAFocus.1.png, FocusPlayerSnapshotTests/testClickControlStates.1.png, FirstRunSnapshotTests/testPracticeScreenStartHere.1.png.
- Rule: T24, "The marker is a fill, with ink on it, on primary buttons, the hero's start button and the swipe under page titles" (docs/design-principles.md:910-913); "It fills primary buttons, the swipe under page titles and the celebration glyph" (:46-49).
- On screen: on one reflection sheet "Partly" is a butter box while the chosen mark is an ink box; a selected answer looks like a primary button.
- Evidence: ChoiceGrid.swift:43, EntryFocusCard.swift:103, FinishRecordSections.swift:108, InstrumentIconPicker.swift:66, ClickControl.swift:217, StartHereCard.swift:107; against SegmentedPills and ScoreSelector.swift:61 (ink).

**I10. A ticked choice has five looks**
- Surface: TagFilterSheet, VariationPickerSheet (plain tick); VariationsSheet, LinkSectionsSheet (ink disc with tick); AddToSessionSheet (ink outline round the whole card plus a disc); AddRelatedExerciseSheet, LinkedItemPickerSheet (plus-circle, in two different inks); ScaffoldPreviewSheet (ink disc beside an unbordered row). Snapshots: LibrarySnapshotTests/testTagFilterSheet.1.png, FocusPlayerSnapshotTests/testVariationPickerSheet.1.png, LibraryDetailSnapshotTests/testVariationsSheetFindingOrAdding.1.png, SessionBuilderSnapshotTests/testAddToSessionSheet.1.png, LibraryDetailSnapshotTests/testAddRelatedExerciseSheet.1.png, LibraryFormSnapshotTests/testLinkedExercisePickerWithDraft.1.png.
- Rule: "Consistency is a simplicity tool. The same gesture means the same thing everywhere" (docs/design-principles.md:117-119); "hand-rolled copies of an existing primitive are the number one source of visual drift" (.claude/rules/ios-ui.md:49-51). `TickRow` is the primitive.
- Evidence: TickRow.swift:18-20; TagFilterSheet.swift:38 and PickerRow.swift:30 (plain tick, used by VariationPickerSheet); SelectableLibraryRow.swift:19-28; ScaffoldPreviewSheet.swift:167-180; LinkedItemPickerSheet.swift:328-333.

**I11. The recovery card's Resume is a system capsule in ink, not the marker button, and the card has no edge**
- Surface: RecoveryPromptCard. Snapshot: PracticeSnapshotTests/testRecoveryPromptCard.1.png.
- Rule: T24 (marker fills primary buttons, docs/design-principles.md:910-913); "Primitives to reach for: ... `.cardSurface()`" (.claude/rules/ios-ui.md:42-46).
- On screen: a black pill "Resume" unlike every other primary action; the white card has no hairline, unlike every other card.
- Evidence: RecoveryPromptCard.swift:31-39 (`.buttonStyle(.borderedProminent)`, `.tint(accent)`), :49-50 (fill and clip, no stroke). Primary buttons come in three shapes overall: 3pt rectangle (MarkerButton, BrandBarButton), marker capsule on the camera ("Use this photo", "Open Settings", PageCameraScreen.swift:253, :299, LibraryFormSnapshotTests/testPageCameraConfirm.1.png), and this ink capsule.

**I12. Five sheets build their own chrome instead of BottomSheet, and lose its large-text title fix**
- Surface: LibraryAddScreen and LibraryEditScreen (ItemFormScaffold), DraftExerciseSheet, ChordChartEditSheet, ProfileEditSheet, FeedbackSheet.
- Rule: "Reuse before creating ... `BottomSheet`" and "Never ship a parallel one-off" (.claude/rules/ios-ui.md:41-51).
- On screen (read off the code; no accessibility-size picture of these five sheets exists): by BottomSheet's own comment, the inline bar truncates a long title at accessibility sizes, which is why BottomSheet moves its title into the content (#2125). These five keep the title in the bar.
- Evidence: BottomSheet.swift:71-74; ItemFormScaffold.swift:43-141, DraftExerciseSheet.swift:14-53, ChordChartEditSheet.swift:38-66, ProfileEditSheet.swift:32, FeedbackSheet.swift:24. Snapshots: LibraryFormSnapshotTests/testLibraryAddScreen.1.png, ProfileSnapshotTests/testProfileEditSheet.1.png, testFeedbackSheet.1.png (DraftExerciseSheet and ChordChartEditSheet are UNCOVERED).

**I13. The chord-chart editor draws its own error box, unlike every other form error**
- Surface: ChordChartEditSheet. Snapshot: UNCOVERED.
- Rule: `FormErrorBanner` is the primitive (.claude/rules/ios-ui.md:44-45, :49-51).
- On screen (read off the code): a filled circle glyph and ink text on the danger wash with no edge, where FormErrorBanner uses the triangle, danger text and a danger edge.
- Evidence: ChordChartEditSheet.swift:114-131 against FormErrorBanner.swift.

**I14. "Remove from this session" is a hand-made delete button, without the trash glyph the other deletes carry**
- Surface: EntrySettingsSheet. Snapshots: SessionBuilderSnapshotTests/testEntrySettingsSheetPopulated.1.png against LibraryDetailSnapshotTests/testSectionSheetEditingATrickySpot.1.png ("Remove section", with the bin).
- Rule: `DeleteButton` is "the default style for an in-content delete" (DeleteButton.swift:3-5); .claude/rules/ios-ui.md:49-51.
- Evidence: EntrySettingsSheet.swift:270-284.

**I15. "Chord chart" is a 17pt ink heading on the add form and a 15pt grey label on the detail screen**
- Surface: LibraryAddScreen (StagedChartCard, the chord chart row), LibraryDetailScreen. Snapshots: LibraryFormSnapshotTests/testStagedChartCardAtFault.1.png against LibraryDetailSnapshotTests/testLibraryDetailChordChartCard.1.png.
- Rule: T35, "A section's title, a field's label and a toggle's title are all 15 point medium in secondary ink" (docs/design-principles.md:1170-1173).
- Evidence: FormSectionRow.swift:19-21 (`cardTitle`, ink), used at StagedChartCard.swift:17 and LibraryAddScreen.swift:98.

**I16. "00:00 left" in the player sets the word "left" in the figures typeface**
- Surface: FocusPlayerScreen section clock. Snapshot: FocusPlayerSnapshotTests/testFocusPlayerSectionTimeUp.1.png.
- Rule: T35, "Mono is for figures. A clock, a duration or a count set alone; a line that mixes words and numbers is Hanken with even-width figures" (docs/design-principles.md:1185-1186).
- Evidence: SegmentClockRow.swift:33-35.

**I17. The kind dots on the Up next card are colour alone**
- Surface: UpNextHero. Snapshots: PracticeSnapshotTests/testUpNextHeroFilledPlan.1.png, testPracticeScreenSuggestionPriorities.1.png.
- Rule: T3, "Colour is always an accelerator, never the sole carrier of meaning. Every type signal must also carry a shape or text cue" (docs/design-principles.md:152-154).
- On screen: a sand or blue-grey 7pt dot is the only sign that "Guide tones" is an exercise and "Like Someone in Love" a piece. VoiceOver gets the word (UpNextHero.swift:280); sighted users do not.
- Evidence: UpNextHero.swift:170-173.

**I18. Renaming a variation is reachable only by a long press**
- Surface: VariationsSheet. Snapshot: UNCOVERED (the menu); the sheet is LibraryDetailSnapshotTests/testVariationsSheet.1.png, which shows no rename affordance.
- Rule: T4, "no action may be reachable only through a hidden gesture" (docs/design-principles.md:159-163).
- Evidence: VariationsSheet.swift:123-126 (context menu plus a VoiceOver action; no visible control). No other rename path exists in `ios/Intrada/Views`.

**I19. The Progress week bars have a 5pt corner and a glow, against the 3pt house corner**
- Surface: AnalyticsScreen, ConsistencyBars. Snapshot: ProgressSnapshotTests/testConsistencyBars.1.png.
- Rule: "Never ... `cornerRadius: 12`" (.claude/rules/ios-ui.md:37-39); "white cards with a 3pt corner and a faint lift" (docs/design-principles.md:38-40); T5's ornament test (:170-174) for the glow.
- Evidence: ConsistencyBars.swift:23 (`cornerRadius: 5`), :31 (`.glow` on the current week).

**I20. Flat and sharp signs render with a gap, as "D ♭ major"**
- Surface: everywhere a key shows: Library cards, detail Key row, Keys wheel, key picker, variation picker, Progress keys. Snapshots: LibrarySnapshotTests/testLibraryScreenPopulated.1.png, LibraryDetailSnapshotTests/testKeysSheetMixed.1.png, FocusPlayerSnapshotTests/testVariationPickerSheet.1.png, ProgressSnapshotTests/testProgressScreenNothingThisWeek.1.png.
- Rule: "Hanken Grotesk for every word, composer and key included" (docs/design-principles.md:53-55); notation "`D♭ major`" (docs/tone-of-voice.md:84-85).
- On screen: the flat is a thin glyph from another face with a visible space each side, so "B♭" reads "B ♭". The crotchet in "♩ = 96" shows the same uneven gap beside "=" (FocusPlayerSnapshotTests/testClickControlDraggingStates.1.png).
- Evidence: the strings come from the core's `keyLabel`, shown as is (LibraryItemView+Display.swift:125, `var keyDisplay: String? { keyLabel }`); the bundle carries only Hanken and DM Mono (ios/Intrada/Resources/Fonts). I did not open the font tables, so "Hanken has no ♭" is inferred from the pictures.

**I21. The middle-dot separator after a section title has a double gap: "Used in  · 3 pieces"**
- Surface: SectionHeader with a caption: Used in, Related exercises, Variations on the exercise detail. Snapshots: LibraryDetailSnapshotTests/testUsedInCardRowStates.1.png, testRelatedExercisesWithSectionLinks.1.png, testExerciseDetailWithVariations.1.png.
- Rule: "The separator on screen is a middle dot with spaces" (docs/tone-of-voice.md:119-121).
- Evidence: SectionHeader.swift:63 (`captionView("· \(caption)")` inside an `HStack` with default spacing).

**I35. "♩ = 132" breaks across two lines, leaving the crotchet orphaned**
- Surface: any meta line carrying a tempo at larger text: Library cards, Add to session rows. Snapshot: SessionBuilderSnapshotTests/testAddToSessionSheetLargestStandardSizeLongKeyAndTempo.1.png ("... espressivo · ♩" on one line, "= 132" on the next).
- Rule: notation reads "where a musician would write it: `♩ = 132`" (docs/tone-of-voice.md:84-85); a broken marking is not that.
- Evidence: crates/intrada-core/src/domain/tempo_words.rs:30 (`format!("{glyph} = {bpm}")` with ordinary spaces, so the line may break at either). Non-breaking spaces there would hold it together.

**I36. Cards have an edge but no lift, though the visual principles promise "a faint lift"**
- Surface: every white card (Library rows, detail cards, form cards, session cards). Snapshots: LibrarySnapshotTests/testLibraryScreenPopulated.1.png, LibraryDetailSnapshotTests/testLibraryDetailScreen.1.png.
- Rule: "white cards with a 3pt corner and a faint lift" (docs/design-principles.md:38-40; T24 at :906); `.cardShadow()` is in the primitive list (.claude/rules/ios-ui.md:44).
- On screen: cards sit flat on the paper, separated only by a hairline.
- Evidence: CardSurface.swift (fill, clip and hairline, no shadow, unchanged since the #1676 reskin); `.cardSurface()` is called on 67 lines in `ios/Intrada/Views`, `.cardShadow()` at four sites only (FocusPlayerScreen.swift:305, KeyWheel.swift:59, AwayOfferBanner.swift:40, BrowseControlsBar.swift:68). No later decision swaps the lift for the edge, so either the primitive or the principle is out of date.

**I37. The builder's edit mode shows the system's bright red delete discs**
- Surface: SessionBuilderScreen in Edit. Snapshot: SessionBuilderSnapshotTests/testSessionBuilderGroupedEditing.1.png.
- Rule: "Warmth bias in semantics. Danger is a brick red" and "One bright colour" (docs/design-principles.md:46-52); `IntradaColor.danger` is the destructive colour (Theme.swift:43).
- On screen: four saturated system-red minus discs down the left, the only pure red in the app, beside the brick-red minus circles used for removal everywhere else.
- Evidence: SessionBuilderScreen.swift:238-279 (a `List` with `.onDelete` and `\.editMode`, so the system draws the discs).

**I38. The Powder highlighter is almost the piece-badge colour, so with it chosen a piece badge reads as a button**
- Surface: TypeBadge, the piece kind bar, any marker-filled control, under the Powder swatch. Snapshots: ProfileSnapshotTests/testHighlighterColours.1.png (powder) against LibrarySnapshotTests/testTypeBadges.1.png (piece).
- Rule: T24, "Type colours are duller pastels than the marker, so a piece badge never reads as a button" (docs/design-principles.md:916-917).
- Evidence: Theme.swift:39 (powder `0xC7DCE8`) against :58 (`pieceBadgeBg` `0xCBD6E0`). The eight swatches (#1677) arrived after T24 set this rule against butter alone.

#### Low

**I39. Validation messages state the limit instead of what is wrong**
- Surface: LibraryAddScreen and LibraryEditScreen error banner. Snapshots: LibraryFormSnapshotTests/testLibraryAddScreenMarksTheFieldAtFault.1.png ("Composer must be between 1 and 200 characters" over an empty composer), testLibraryAddScreenMarksTheStagedRowInContext.1.png.
- Rule: rule 8, "Say what happened and what it means for the user's data" (docs/tone-of-voice.md:157-160).
- Evidence: crates/intrada-core/src/validation.rs:144, :162, :207, :264. An empty field would read better as "Add a composer".

**I40. Two standalone lines start in lower case**
- Surface: MasteryDelta rows on Progress; the "Last five weeks" caption. Snapshots: ProgressSnapshotTests/testMasteryDeltaRows.1.png ("first time marked"), testProgressScreenNothingThisWeek.1.png ("best week · 95m").
- Rule: "Sentence case everywhere" (docs/tone-of-voice.md:123-125).
- Evidence: crates/intrada-core/src/analytics.rs:71, :339.

**I22. The summary's session-note field hand-copies `.cardSurface()`; three other surfaces do the same**
- Rule: .claude/rules/ios-ui.md:42-46, :49-51.
- Evidence: SessionSummaryScreen.swift:281-286 (exact copy), TempoStepButton.swift:19-23, AwayOfferBanner.swift:35-38, SegmentClockRow.swift:56-61. Snapshots: SessionSummarySnapshotTests/testSessionSummaryCompleted.1.png, FocusPlayerSnapshotTests/testFocusPlayerAwayOffer.1.png. Code-only drift; the pictures match.

**I23. Two hand-made badges beside `TagChip` and `TypeBadge`**
- Surface: ScaffoldPreviewSheet ("Already added", "Fallback"), SessionBuilderScreen ("Group"). Snapshots: LibraryDetailSnapshotTests/testScaffoldPreviewSheet.1.png, SessionBuilderSnapshotTests/testSessionBuilderGrouped.1.png.
- Rule: .claude/rules/ios-ui.md:42, :49-51.
- Evidence: ScaffoldPreviewSheet.swift:194-207; SessionBuilderScreen.swift:531-533.

**I24. Three dash patterns for dashed outlines, and the empty-day card borrows the add-slot look though it does nothing**
- Surface: PracticeScreen empty day, AddRowButton, OfferChip, From your note chips. Snapshots: PracticeSnapshotTests/testPracticeScreenQuietDay.1.png, LibraryDetailSnapshotTests/testLibraryDetailScreen.1.png, FocusPlayerSnapshotTests/testFocusPlayerLastTimeOffer.1.png.
- Rule: one pattern per job (docs/design-principles.md:117-119); AddRowButton says the dash "reads as an empty slot inviting input" (AddRowButton.swift:4-5).
- Evidence: PracticeScreen.swift:402-407 (`slotOutline`, dash `[5]`, not tappable); AddRowButton.swift:45-46 (`[4, 4]`); OfferChip.swift:33-34 and FinishRecordSections.swift:110-111 (`[4, 3]`).

**I25. A chord chart is SF Mono on the add form and in the editor, Hanken on the detail card**
- Surface: StagedChartCard, ChordChartEditSheet, ChordChartCard. Snapshots: LibraryFormSnapshotTests/testStagedChartCardAtFault.1.png against LibraryDetailSnapshotTests/testLibraryDetailChordChartCard.1.png.
- Rule: "Hanken Grotesk for every word ...; DM Mono only for a value that is a number alone" (docs/design-principles.md:53-55); typography through the five sizes (.claude/rules/ios-ui.md:52-59).
- Evidence: Theme.swift:192-193 (`Font.system(... design: .monospaced)`), StagedChartCard.swift:29, ChordChartEditSheet.swift:90, :97, :140.

**I26. Synthetic bold on the custom font**
- Rule: "weights use named-instance PostScript names, not `.weight()`, which is synthetic over a variable axis" (Theme.swift:150-151).
- Evidence: RepCounter.swift:160, :165 ("Got it", "Not quite"), FocusPlayerScreen.swift:298, PickerRow.swift:31. Snapshot: FocusPlayerSnapshotTests/testRepCounter.1.png.

**I27. A system text style sizes the add and tick glyphs on the selectable library row**
- Rule: "SF Symbols through `IntradaIconSize`" (.claude/rules/ios-ui.md:61).
- Evidence: SelectableLibraryRow.swift:20 (`.font(.title2)`). Snapshot: SessionBuilderSnapshotTests/testAddToSessionSheet.1.png.

**I28. About forty literal paddings outside the spacing tokens, some repeating a token's value**
- Rule: "Never ... `.padding(16)`" (.claude/rules/ios-ui.md:37-40).
- Evidence (the patterns, not every line): `.padding(.horizontal, 16)` at MasteryDelta.swift:102 (that is `IntradaSpacing.card`); `.padding(.leading, 20)` four times (LibraryItemCard.swift:76, RelatedExercisesCard.swift:250, DraftItemRow.swift:42, ScaffoldPreviewSheet.swift:151), a missing token; `.padding(.vertical, 10)` seven times across the field primitives (FieldCard.swift:41, FormField.swift:38, AutocompleteField.swift:41, TagChipInput.swift:42, InlineSuggestionList.swift:29, KeyPicker.swift:95, ScanPageEntry.swift:113); `19` twice at SessionSummaryScreen.swift:163, :253; `18` and `14` in the player (FocusPlayerScreen.swift:301, :630); `13`, `11`, `14` at RepCounter.swift:168, MasteryDelta.swift:45-46. Code-only drift.

**I29. Faint tokens on things that carry meaning or sit as text**
- Rule: `inkFaintIcon` is for "a dimmed glyph with no text of its own ... Never for text" (Theme.swift:16-18); WCAG 1.4.11's 3:1 for meaningful graphics.
- Evidence: TempoTrend.swift:169-172 (the "not measured" ticks in `inkFainter`, well under 3:1, LibraryDetailSnapshotTests/testTempoTrend.1.png); KeyPicker.swift:140-142 (a `Text("♪")` in `inkFaintIcon`, LibraryFormSnapshotTests/testKeyPickerExpandedEmpty.1.png); ListDetailSplit.swift:29 (`inkFainter` placeholder glyph).

**I30. The mastery toast fades the earlier mark instead of stepping its colour**
- Surface: MasteryDeltaToast. Snapshot: ProgressSnapshotTests/testProgressScreenPopulated.1.png ("2 → 3").
- Rule: T31, "The earlier score in a mastery change ... take `inkSecondary`; the paler colours they used are deleted" (docs/design-principles.md:1087-1088).
- Evidence: MasteryDelta.swift:91-92 (`.opacity(IntradaOpacity.dimmed)`). On the dark toast there is no `inkSecondary` equivalent, so this may want a token rather than a fix.

**I31. Possessives on chrome**
- Surface: FirstRunScreen titles, LinkedItemPickerSheet empty state, profile name placeholder. Snapshots: FirstRunSnapshotTests/testProfileStep.1.png, testFirstPieceStep.1.png.
- Rule: rule 3, "No possessives on chrome: `Library`, not `Your Library`"; V2's test, "If it points at a screen, cut it" (docs/tone-of-voice.md:92-94, :321-333).
- Evidence: FirstRunScreen.swift:254 ("Your profile"), :316 ("Your first piece"); LinkedItemPickerSheet.swift:377 ("No pieces in your library yet."); ProfileFields.swift:13 (placeholder "Your name"). "Your profile" points at something the musician is about to author, so this is arguable.

**I32. The first-run screen draws an exercise as a repeat arrow; everywhere else it is a dumbbell**
- Surface: FirstRunScreen first-piece step. Snapshot: FirstRunSnapshotTests/testFirstPieceStep.1.png.
- Rule: T29, "one shape means what kind of thing this is" (docs/design-principles.md:1056-1058).
- Evidence: FirstRunScreen.swift:339 (`"repeat"`) against ItemKindStyle.swift:27-30 (`dumbbell.fill`).

**I33. The Routines tab promises routines and offers to build a session**
- Surface: RoutinesScreen. Snapshot: ProgressSnapshotTests/testRoutinesScreen.1.png.
- Rule: rule 7, an empty state is "what belongs here, what to do, and the action itself" (docs/tone-of-voice.md:146-150).
- Evidence: RoutinesScreen.swift:9-14. Known: docs/roadmap.md:52-53 (v0.21.0 decides the fourth tab), so this is a copy note until then.

**I34. The Practice header no longer carries "Last practised", which T25 still says it does**
- Surface: PracticeScreen. Snapshots: PracticeSnapshotTests/testPracticeScreenPopulated.1.png, testPracticeScreenGreeting.1.png.
- Rule: T25, "the last-practised fact stays after it: `Morning, Jon · Last practised Tuesday`" (docs/design-principles.md:924-928); tone V6 (docs/tone-of-voice.md:384-393).
- On screen: the header reads "Morning, Jon" alone; "Last practised · Yesterday" moved into the hero.
- Evidence: PracticeScreen.swift:290-294, :423-432 (comment cites #1725). The screen is probably right and the log stale; either amend T25 and V6 or move the line back.

### (c) Refinement observations (not defects)

1. **Library rows are one white block after another.** Three cards with a 2pt kind bar, a grey eighth-rest ring and the same rhythm (testLibraryScreenPopulated). The ring for "never marked" reads as an empty clock rather than a state; the kind bar is the only colour. A denser row or a stronger kind cue would help scanning.
2. **The piece detail page is a stack of equal-weight cards.** Notes, Sections, Variations, Practise in other keys, Key and Tempo, Photo, Chord chart, each white with a grey 15pt label (testLibraryDetailScreen). Nothing says which is the content and which is admin; three different "add" looks (dashed with hint, plain row in a card, "Add" text action) sit one after another.
3. **The session detail is flat beside the summary.** Rows of 17pt title over 13pt meta with a ring, no cards, under a single carded "How it went" (testPracticeSessionDetail). "How it went" and "Your mark for the session" are both grey, so the card has no lead line.
4. **Practice repeats itself.** Section header "Yesterday · 1 session" sits over a card titled "Yesterday" (testPracticeScreenPopulated); with Start here showing, the hero's play button and the "Build a session" row are the same action twice (testPracticeScreenStartHere).
5. **The player's middle is crowded with equal-weight capsules.** Variation chip, last-time chip, trouble-spot button and metronome readout each take a different capsule style (testFocusPlayerWithVariations). The rep count's right edge stops 50pt short of the buttons below it because a hidden Undo reserves the space (RepCounter.swift:76-88), which reads as misalignment.
6. **Solid-stroke capsules show a short vertical line at each end in the snapshots** (trouble spot, star filter, search field, kept note chips: testFocusPlayerWithTarget, testLibraryScreenPopulated, testLibraryScreenSearchMatchesNothing, testReflectionSheetWithFinishAnswers). Dashed and unstroked capsules do not. Likely a renderer artefact; worth one look on a device before it is dismissed.
7. **Sheets left mostly empty at full height.** Trouble spot, Switch variation, Feedback and the empty tag filter open at `.large` with three-quarters of the sheet blank (testTroubleSpotSheet, testVariationPickerSheet, testFeedbackSheet). The trouble-spot wheels are two bare white slabs with square corners.
8. **Secondary actions have three looks.** Boxed outline ("Change" on the profile icon, ProfileEditSheet.swift:120-127), sunken chip ("Link", CapsuleActionButton), and plain ink text ("Add", "Edit", "Choose", "Reorder"). One quiet-action style would calm the detail screens.
9. **Durations are written five ways.** "32m", "12 min", "95m", "20m 50s", "6h 20m" across Practice, the builder, Progress and the summary. No rule names a format yet.
10. **Profile is monotone.** One grey label per card and a single "Send feedback" row under "Beta" (testProfileScreenFull); the empty state's "Add a name" at title size in grey reads as a disabled heading rather than an invitation (testProfileScreenEmpty).
11. **Explanatory footnotes stack up on sheets.** "Fallback flags a change mapped to its arpeggio, not a scale.", "Tap a key to add it. Tap a chosen key with ⇅ to switch its spelling; tap again to remove it.", "A new variation joins the list on every piece and exercise." Tone doc: "A screen that needs a paragraph to explain itself is a design problem" (docs/tone-of-voice.md:37-39).
12. **Remove and reorder controls swap sides from list to list.** Sections in reorder mode: minus left, grip right; variations on the form: grip left, minus right; related exercises: arrows then minus on the right; builder in Edit: system disc left, grip right (testSectionsReordering, testLibraryEditScreenExerciseWithVariations, testRelatedExercisesSectionLinksEditing, testSessionBuilderGroupedEditing).
13. **Quiet text actions use two inks.** "Change" and "Done" on the reflection sheet, "Discard", "Show suggestion" and "Skip" are `inkSecondary`; "Edit", "Add", "Choose" and "Reorder" are ink (ReflectionSheet.swift:388-389, FinishRecordSections.swift:33-35). Probably deliberate demotion, but there is no rule saying which actions get which.
14. **The Variations sheet repeats search matches.** While finding "Left hand", the two matches show in the results card and again in "All variations" below (testVariationsSheetFindingOrAdding).

## Android design audit (issue #2599)

Read-only audit of `android/app/src/main/kotlin/com/intrada/android/ui/` against the iPhone design, at `e3cd2990` in the worktree `design-audit-2599`. Nothing was built or run. Every Android snapshot named below was looked at beside its iOS pair where one exists; every finding cites the Kotlin line it rests on.

Paths are shortened: `A/` is `android/app/src/test/snapshots/`, `I/` is `ios/IntradaTests/__Snapshots__/`, `K/` is `android/app/src/main/kotlin/com/intrada/android/ui/`.

### (a) Inventory

No Android snapshot test opens a dialog or menu (no `confirming*`, `options`, `menuFor` or viewer flag is set in any `*SnapshotTest.kt`), so every dialog and menu is UNCOVERED. iOS confirm dialogs are native `.alert` calls with no snapshot.

| Surface | Kind | Android snapshot | iOS counterpart snapshot |
|---|---|---|---|
| Tab bar and frame (`AppFrame.kt` TabBar) | chrome | A/frame-library.png, A/frame-practice.png, A/frame-progress.png, A/frame-routines.png | I/RootShellSnapshotTests/testRootShell.1.png |
| Library (`LibraryScreen.kt`) | screen | A/library.png, A/library-error.png | I/LibrarySnapshotTests/testLibraryScreenPopulated.1.png, testLibraryScreen.1.png |
| Library empty state | state | UNCOVERED (empty list path, LibraryScreen.kt:83) | I/RootShellSnapshotTests/testRootShell.1.png |
| Library item card (`LibraryItemCard`) | component | A/library.png | I/LibrarySnapshotTests/testLibraryItemCards.1.png |
| Routines (`EmptyTab`) | screen | A/frame-routines.png | I/ProgressSnapshotTests/testRoutinesScreen.1.png |
| Practice (`PracticeScreen.kt`) | screen | A/practice-empty.png, A/practice-filled.png, A/frame-practice.png | I/PracticeSnapshotTests/testPracticeScreen.1.png, testPracticeScreenPopulated.1.png |
| Last practised hero (`LastPractisedHero`) | card | A/practice-filled.png | I/PracticeSnapshotTests/testPracticeScreenPopulated.1.png |
| Up next card (`UpNextCard.kt`) | card | A/practice-up-next.png, A/practice-up-next-filled.png, A/practice-up-next-dismissed.png | I/PracticeSnapshotTests/testUpNextHeroFilledPlan.1.png, testPracticeScreenSuggestion.1.png, testPracticeScreenSuggestionDismissed.1.png |
| Start here card (`StartHereCard.kt`) | card | A/start-here-add.png, A/start-here-build.png, A/start-here-play.png, A/start-here-mark.png | I/FirstRunSnapshotTests/testPracticeScreenStartHere.1.png |
| Week strip, session card, empty day card (`PracticeParts.kt`) | component | A/practice-filled.png, A/frame-practice.png | I/ComponentSnapshotTests/testWeekStripDimsDaysStillToCome.1.png, I/PracticeSnapshotTests/testSessionCardsPlayedSummary.1.png |
| Session detail (`SessionDetailScreen.kt`) | screen | A/session-detail.png, A/session-detail-variations.png | I/PracticeSnapshotTests/testPracticeSessionDetail.1.png, testPracticeSessionDetailWithVariations.1.png |
| Recovery card (`RecoveryCard.kt`) | card | A/player-recovery.png | I/PracticeSnapshotTests/testRecoveryPromptCard.1.png |
| Build session (`BuilderScreen.kt`) | screen | A/builder.png, A/builder-empty.png, A/builder-editing.png | I/SessionBuilderSnapshotTests/testSessionBuilderPopulated.1.png, testSessionBuilderEmpty.1.png, testSessionBuilderGroupedEditing.1.png |
| Block menu (`BlockMenu`) | menu | UNCOVERED | none (native Menu) |
| Discard plan (`DiscardDialog`) | dialog | UNCOVERED | none (`.alert`) |
| Add to session (`LibraryPickerScreen`) | sheet | A/builder-add.png | I/SessionBuilderSnapshotTests/testAddToSessionSheet.1.png |
| Add related exercise (`AddRelatedRoute`) | sheet | UNCOVERED | I/LibraryDetailSnapshotTests/testAddRelatedExerciseSheet.1.png |
| Entry settings (`EntrySettingsScreen.kt`, `EntryFocusCard.kt`) | sheet | A/builder-entry-settings.png | I/SessionBuilderSnapshotTests/testEntrySettingsSheetPopulated.1.png, testEntrySettingsSheetSuggestsAFocus.1.png |
| Player (`PlayerScreen.kt`) | screen | A/player.png, A/player-record.png | I/FocusPlayerSnapshotTests/testFocusPlayerWithReps.1.png, testFocusPlayerSectionTimeUp.1.png |
| Player options (`OptionsDialog`) | menu | UNCOVERED | none (native Menu) |
| Time away offer (`AwayOffer`) | banner | UNCOVERED | I/FocusPlayerSnapshotTests/testFocusPlayerAwayOffer.1.png |
| Click control and bar (`ClickRow.kt`, `ClickBar.kt`, `ClickBarLine.kt`) | component | A/player-click.png, A/player-click-bar.png | I/FocusPlayerSnapshotTests/testClickControlStates.1.png, testClickBarLineStates.1.png |
| Metronome sheet (`ClickSheet.kt`) | sheet | A/player-click-sheet.png, A/player-click-sheet-long.png | I/FocusPlayerSnapshotTests/testClickSheetIrregularMetre.1.png, testClickSheetLongBar.1.png |
| Variation picker (`RecordSheets`) | sheet | A/player-variation-picker.png | I/FocusPlayerSnapshotTests/testVariationPickerSheet.1.png |
| Trouble spot (`RecordSheets`) | sheet | A/player-trouble-spot.png | I/FocusPlayerSnapshotTests/testTroubleSpotSheet.1.png |
| Reflection (`ReflectionSheet.kt`) | sheet | A/player-reflection.png, A/player-reflection-record.png | I/SessionSummarySnapshotTests/testReflectionSheet.1.png, testReflectionSheetWithFinishAnswers.1.png |
| Session summary (`SummaryScreen.kt`) | screen | A/player-summary.png, A/player-summary-note.png | I/SessionSummarySnapshotTests/testSessionSummaryCompleted.1.png, testSessionSummaryEndedEarly.1.png |
| Discard session (summary `ConfirmDialog`) | dialog | UNCOVERED | none (`.alert`) |
| Piece (`PieceScreen.kt`) | screen | A/piece.png, A/piece-empty.png, A/piece-editing.png | I/LibraryDetailSnapshotTests/testLibraryDetailScreen.1.png, testPieceDetailLinkedEmpty.1.png, testPieceDetailLinkedEditing.1.png |
| Delete piece (`ConfirmDialog`) | dialog | UNCOVERED | none (`.alert`) |
| Section (`SectionScreen.kt`) | sheet | A/piece-section.png | I/LibraryDetailSnapshotTests/testSectionSheetEditingATrickySpot.1.png |
| Remove section (`ConfirmDialog`) | dialog | UNCOVERED | none (`.alert`) |
| Link sections (`LinkSectionsScreen.kt`) | sheet | A/piece-link-sections.png | I/LibraryDetailSnapshotTests/testLinkSectionsSheet.1.png |
| Exercise picker (`ExercisePickerScreen.kt`) | sheet | A/piece-exercise-picker.png | I/LibraryDetailSnapshotTests/testAddRelatedExerciseSheet.1.png |
| Keys sheet (`PieceKeys.kt`) | sheet | A/piece-keys-sheet.png | I/LibraryDetailSnapshotTests/testKeysSheetMixed.1.png |
| Variations sheet (`PieceVariations.kt`) | sheet | A/piece-variations-sheet.png | I/LibraryDetailSnapshotTests/testVariationsSheet.1.png |
| Rename variation (`RenameDialog`) | dialog | UNCOVERED | none |
| Photo card (`PhotoCard.kt`) | card | A/piece-photo.png | I/LibraryDetailSnapshotTests/testPhotoCardStates.1.png |
| Photo viewer (`PhotoViewer`) | dialog | UNCOVERED | none |
| Photo capture menu (`CaptureMenu`) | menu | UNCOVERED | none |
| Remove photo (`ConfirmDialog`) | dialog | UNCOVERED | none (`.alert`) |
| Page camera (`PageCapture.kt`) | screen | UNCOVERED | I/LibraryFormSnapshotTests/testPageCameraStates.1.png |
| Item form (`ItemFormScreen.kt`) | screen | A/item-form-add.png, A/item-form-add-refused.png, A/item-form-edit-exercise.png | I/LibraryFormSnapshotTests/testLibraryAddScreen.1.png, testLibraryAddScreenWithError.1.png, testLibraryEditScreenExercise.1.png |
| Key picker and wheel (`KeyWheel.kt`) | component | A/item-form-key-wheel.png | I/LibraryFormSnapshotTests/testKeyPickerExpandedEnharmonic.1.png |
| Composer suggestions (`SuggestionRows.kt`) | component | A/item-form-composer-suggestions.png | I/LibraryFormSnapshotTests/testAutocompleteField.1.png |
| Related exercises on the form (`ItemFormExercises.kt`) | sheet | A/item-form-related-exercises.png | I/LibraryFormSnapshotTests/testLinkedExercisePicker.1.png |
| Write an exercise (`WrittenExerciseDialog`) | dialog | UNCOVERED | none |
| Remove variation (`VariationRow.kt` `ConfirmDialog`) | dialog | UNCOVERED | none (`.alert`) |
| Scan a page card (`ScanPageEntry.kt`) | card | A/scan-page-states.png | I/LibraryFormSnapshotTests/testScanPageEntryStates.1.png |
| Profile (`ProfileScreen.kt`) | screen | A/profile.png, A/profile-empty.png | I/ProfileSnapshotTests/testProfileScreen.1.png, testProfileScreenEmpty.1.png |
| Profile edit (`ProfileEditSheet.kt`) | sheet | A/profile-edit.png, A/profile-edit-refused.png | I/ProfileSnapshotTests/testProfileEditSheet.1.png, testProfileEditSheetWithError.1.png |
| Instrument icon picker (`ProfileParts.kt`) | component | A/profile-icon-picker.png | I/ProfileSnapshotTests/testInstrumentIconPicker.1.png |
| Feedback (`FeedbackSheet.kt`) | sheet | A/feedback-sheet.png | I/ProfileSnapshotTests/testFeedbackSheet.1.png |
| Progress (`ProgressScreen.kt`, `ProgressParts.kt`) | screen | A/progress.png, A/progress-empty.png, A/progress-mastery-fall.png | I/ProgressSnapshotTests/testProgressScreenPopulated.1.png, testProgressScreenNothingThisWeek.1.png, testMasteryDeltaRowAfterAFall.1.png |
| First run: welcome, profile, first piece (`FirstRunScreen.kt`) | screen | A/first-run-welcome.png, A/first-run-profile.png, A/first-run-first-piece.png | I/FirstRunSnapshotTests/testWelcome.1.png, testProfileStep.1.png, testFirstPieceStep.1.png |
| Global banner (`GlobalBanner.kt`) | banner | A/banner.png | I/RootShellSnapshotTests/testGlobalBanner.1.png |
| Form error banner (`components/FormErrorBanner.kt`) | banner | A/form-error-banner.png | none on its own (inside I/LibraryFormSnapshotTests/testLibraryAddScreenWithError.1.png) |
| Missing item (`MissingItem`) | screen | UNCOVERED | none |
| Primitives: card surface, card shadow, hairline, scrim capsule, tag chip, instrument glyph, profile badge, field card (`components/`) | component | A/card-surface.png, A/card-shadow.png, A/hairline-divider.png, A/scrim-capsule.png, A/tag-chip.png, A/instrument-glyph.png, A/profile-badge.png, A/field-card.png | I/LibraryFormSnapshotTests/testTagChipInput.1.png, I/ProfileSnapshotTests/testInstrumentIcons.1.png; none for the rest |

UNCOVERED: 18 (library empty state, block menu, discard plan, add related exercise, player options, time away offer, discard session, delete piece, remove section, rename variation, photo viewer, photo capture menu, remove photo, page camera, write an exercise, remove variation, missing item, and the launch splash, which has no Android surface at all).

**Not yet built, kept out of the findings:** the browse bar (priority star, kind filter, sort, search) on Library and Add to session, the tag filter sheet, the launch splash (T34), speech input on the reflection note, the chord chart card and the "Practise this" button on a piece. These are unbuilt scope under #2220, not drift.

### (b) Findings

Severity: high is wrong or inaccessible; medium is visible drift from iOS or the design; low is code only.

#### High

**A1. Notices the core raises are never shown**
- Surface: every screen (banner stack). Android A/banner.png; iOS I/RootShellSnapshotTests/testGlobalBanner.1.png.
- Rule: T30, "Notice is for something that worked but needs a word: `inkSecondary` on `cardFill` with a hairline underneath and the info circle" (docs/design-principles.md:1068); and android-quality.md:89, "Surface, don't swallow."
- On screen: iOS shows a white banner with an info circle under the red one ("That metronome setting doesn't give a crotchet tempo, so this play has none."). Android has only the red banner, so that sentence never appears at all.
- Evidence: K/GlobalBanner.kt:44 hard-codes `danger` ink and K/GlobalBanner.kt:84 the warning triangle; no Kotlin file reads the view model's notice (`crates/intrada-core/src/model.rs:295`, shown on iOS at `ios/Intrada/Views/Components/AppBannerStack.swift:18`). No info-circle drawable exists in `res/drawable`.

**A2. The switch's off state is invisible**
- Surface: Build session length, Profile practice defaults, Reflection "Add detail", entry settings toggles. Android A/builder.png, A/profile.png, A/player-reflection.png; iOS I/ProfileSnapshotTests/testProfileScreen.1.png.
- Rule: T24, "a pastel border on white misses the 3:1 WCAG sets for a control" (docs/design-principles.md:917), which is the same 3:1 non-text floor.
- On screen: an off switch is a white dot on a track the colour of the paper; on the reflection page it reads as a faint smudge rather than a control. Profile "Session length" looks the same.
- Evidence: K/SettingControls.kt:64 paints the off track `surfaceSunken` (#F3EFE8) on `paperTop` (#F7F4EF) or white, with a white thumb at :68.

**A3. Two filled buttons compete on the player when a section's time is up**
- Surface: Player with a section clock. Android A/player-record.png; iOS I/FocusPlayerSnapshotTests/testFocusPlayerSectionTimeUp.1.png.
- Rule: Interaction principle A, "One primary action per screen. Each screen has one obvious next step, visually dominant" (docs/design-principles.md:88).
- On screen: Android shows a full-width black "Move on to Run" bar and, lower down, a second full-width black "Finish session" bar of the same weight. iOS has one yellow "On to B" bar and a round play button for the transport.
- Evidence: K/PlayerRecordParts.kt:90 and K/PlayerScreen.kt:333 both use `InkButton`.

#### Medium

**A4. Primary buttons are black, not the musician's highlighter**
- Surface: Build session start, Reflection save, Session summary save, player "Move on", recovery "Resume", trouble spot "Add spot". Android A/builder.png, A/player-reflection.png, A/player-summary.png; iOS I/SessionBuilderSnapshotTests/testSessionBuilderPopulated.1.png, I/SessionSummarySnapshotTests/testReflectionSheet.1.png.
- Rule: T24, "The marker is a fill, with ink on it (11.3:1), on primary buttons" (docs/design-principles.md:911); ios-ui.md:49, "Never ship a parallel one-off".
- On screen: "Start session · 10 min", "Save & continue" and "Save session" are black bars with white text and no glyph; iOS shows yellow bars with a play glyph or a trailing arrow and a faint lift.
- Evidence: `InkButton` at K/PlayerParts.kt:53-81 (fill `IntradaGradient.inkBar` at :65), called at K/BuilderScreen.kt:233, K/ReflectionSheet.kt:250, K/SummaryScreen.kt:132, K/PlayerRecordParts.kt:90, K/RecoveryCard.kt:47, K/PlayerSheets.kt:149. A marker button already exists, private to the first run, at K/FirstRunScreen.kt:348.

**A5. No highlighter swipe under page titles**
- Surface: every screen title. Android A/frame-library.png and every full-screen snapshot; iOS I/LibrarySnapshotTests/testLibraryScreenPopulated.1.png.
- Rule: Visual principles, the highlighter "fills primary buttons, the swipe under page titles and the celebration glyph" (docs/design-principles.md:47).
- On screen: "Library", "Practice", "Progress" sit on plain paper; on iOS each has a yellow (or the chosen colour) band behind its lower half.
- Evidence: K/ScreenScaffold.kt:56-59 draws the title with no background; iOS `ios/Intrada/DesignSystem/MarkerSwipe.swift`.

**A6. The highlighter colour is not passed down, so parts stay butter for everyone**
- Surface: Progress mastery toast, the running metronome readout. Android A/progress.png, A/player-click.png; iOS I/ProfileSnapshotTests/testProfileScreen.1.png (coral title swipe).
- Rule: ios-ui.md:9, "the musician's own choice of eight through `@Environment(\.marker)`".
- On screen: a musician who chose mint or coral still sees a butter circle on the "Mastery up" card and a butter fill on the running click.
- Evidence: K/ProgressParts.kt:305 and :325 use `IntradaColor.marker`; K/ClickRow.kt:211 the same. Android has no composition local for the marker; only Practice and Profile read the profile colour (K/PracticeScreen.kt:79).

**A7. Segmented pills are drawn inside out**
- Surface: Piece or Exercise toggle, metronome time signature, beat value, grouping, "Sounds on", profile "Metronome starts on". Android A/item-form-add.png, A/player-click-sheet.png, A/profile.png; iOS I/LibraryFormSnapshotTests/testLibraryAddScreen.1.png, I/FocusPlayerSnapshotTests/testClickSheetIrregularMetre.1.png.
- Rule: ios-ui.md:49, "If a primitive almost fits, add a parameter to it ... hand-rolled copies of an existing primitive are the number one source of visual drift."
- On screen: iOS has a white rounded track with a hairline and the chosen option as a black capsule with white words. Android has a beige square-cornered track with the chosen option as a white square, so the choice reads faintly.
- Evidence: K/PageParts.kt:326-362 (track `surfaceSunken` at :330, chosen `cardFill` at :338, `IntradaRadius.control` corners at :326); iOS `ios/Intrada/Views/Components/SegmentedPills.swift:96-144`.

**A8. Plus and minus steppers are loose icons**
- Surface: session length, repetitions, beats in the bar, tempo target, profile defaults. Android A/builder.png, A/profile.png, A/player-click-sheet.png; iOS I/ProfileSnapshotTests/testProfileScreen.1.png.
- Rule: ios-ui.md:49 (as A7).
- On screen: iOS joins minus and plus in a grey capsule with a divider. Android shows two bare glyphs floating at the right of the card.
- Evidence: K/SettingControls.kt:102-115.

**A9. The mark picker fills one box, not the run up to it**
- Surface: Reflection and Session summary marks. Android A/player-reflection.png, A/player-summary.png; iOS I/SessionSummarySnapshotTests/testSessionSummaryCompleted.1.png, testScoreSelectorPills.1.png.
- Rule: ios-ui.md:49; T35, "Mono is for figures. A clock, a duration or a count set alone" (a mark in a row of pills is a label, set in the badge style on iOS).
- On screen: iOS shows outlined boxes with every box up to the mark filled black. Android shows beige filled boxes, only the chosen one black, with typewriter digits so "10" shows a slashed zero.
- Evidence: K/PlayerParts.kt:98-121 (`chosen = value == score` at :98, `IntradaFont.figure` at :117); iOS `ios/Intrada/Views/Components/ScoreSelector.swift:49-66`.

**A10. The player's type label is plain text, not the badge**
- Surface: Player. Android A/player.png; iOS I/FocusPlayerSnapshotTests/testFocusPlayerWithReps.1.png.
- Rule: T29, "The badge is a pill wherever it appears, the Practice player included" (docs/design-principles.md:1057); T3, "Every type signal must also carry a shape or text cue".
- On screen: Android shows a small grey "Piece" above the title; iOS a tinted pill with a note or dumbbell glyph.
- Evidence: K/PlayerScreen.kt:266-269. There is no type badge primitive in Kotlin and no note or dumbbell drawable in `res/drawable`.

**A11. The piece page's type pill is hand-rolled and has no glyph**
- Surface: Piece. Android A/piece.png; iOS I/LibraryDetailSnapshotTests/testLibraryDetailScreen.1.png.
- Rule: ios-ui.md:42-46 lists `TypeBadge` as the primitive to reach for; T3 as A10.
- On screen: a plain blue-grey "Piece" capsule; iOS adds the note glyph. Its padding is set by hand.
- Evidence: K/PieceScreen.kt:203-208 (literal `10.dp, 5.dp` at :206).

**A12. The session progress band uses pastel type colours**
- Surface: Player header. Android A/player.png; iOS I/FocusPlayerSnapshotTests/testFocusPlayerWithReps.1.png.
- Rule: ios `ItemKindStyle.swift`: "Ink for both kinds: a pastel border on white misses 3:1 (WCAG 1.4.11)"; T24, "A check or outline stays ink for both kinds".
- On screen: done segments on Android are pale blue-grey or sand against a pale track; on iOS they are black. Android also puts "Focus · 1 of 2" left and the clock right, where iOS has the clock left and the position centred.
- Evidence: K/PlayerParts.kt:361 (`kind.bar`), :329-346 for the order.

**A13. The player transport is a bar and a text link**
- Surface: Player. Android A/player.png; iOS I/FocusPlayerSnapshotTests/testFocusPlayerWithReps.1.png.
- Rule: Interaction principle B, "The app disappears during practice" and T2 ("The default live-session screen stays bare").
- On screen: Android ends with a full-width black "Next item" bar and "Skip this item" below it; iOS has a round black play button and a small skip-forward icon, with far less weight at the bottom of the page.
- Evidence: K/PlayerScreen.kt:326-356; iOS `ios/Intrada/Views/Screens/FocusPlayerScreen.swift:428-452`.

**A14. The rep counter lacks its dots and button glyphs**
- Surface: Player. Android A/player.png, A/player-record.png; iOS I/FocusPlayerSnapshotTests/testFocusPlayerWithReps.1.png.
- Rule: Interaction principle C, "Glanceable, not just readable."
- On screen: iOS shows a row of filled and hollow dots for the reps and a cross and tick on "Not quite" and "Got it". Android has the words only; the count uses typewriter digits ("Ø of 10").
- Evidence: K/PlayerParts.kt:240-292 and RepButton :294-316; count style `IntradaFont.figure` at :265.

**A15. The trouble spot button leads with a plus, not a flag**
- Surface: Player. Android A/player.png; iOS I/FocusPlayerSnapshotTests/testFocusPlayerWithReps.1.png.
- Rule: Interaction principle C, "Consistency is a simplicity tool."
- On screen: "+ Trouble spot" with a tiny plus; iOS uses a flag glyph at body size.
- Evidence: K/PlayerRecordParts.kt:253 (`ic_plus` at caption size).

**A16. The summary headline is display size**
- Surface: Session summary. Android A/player-summary.png; iOS I/SessionSummarySnapshotTests/testSessionSummaryCompleted.1.png.
- Rule: T35, "No screen passes a size; the timer, the tempo readouts and the score and mastery numerals are the only figures sized by the dial or readout they sit in."
- On screen: "6m 0s" is about twice the page title's height, larger than "Session complete". iOS sets it at page title size.
- Evidence: K/SummaryScreen.kt:149 (`IntradaFont.timer()`, 56sp); iOS `SessionSummaryScreen.swift:65` uses `pageTitle`.

**A17. The summary boxes its items instead of listing them**
- Surface: Session summary. Android A/player-summary.png; iOS I/SessionSummarySnapshotTests/testSessionSummaryCompleted.1.png.
- Rule: T35, "A section's title ... at the top of what they name"; consistency with Session detail, which Android already lays out the iOS way (A/session-detail.png).
- On screen: iOS has a "What you played" label and each item on the paper with a dot and a divider. Android puts them in one white card with no section label and no dots.
- Evidence: no "What you played" or dot in K/SummaryScreen.kt (grep finds none); the card wraps the rows.

**A18. Add rows are solid cards, not dashed slots**
- Surface: Build session "Add piece or exercise", piece "Add section". Android A/builder.png, A/piece.png; iOS I/SessionBuilderSnapshotTests/testSessionBuilderPopulated.1.png, I/LibraryDetailSnapshotTests/testLibraryDetailScreen.1.png.
- Rule: ios-ui.md:49 (as A7); iOS `AddRowButton` "`.dashed` (default) reads as an empty slot inviting input".
- On screen: a white card reading "+ Add piece or exercise"; iOS a dashed outline with a plus glyph and the label beside it. The "+" on Android is a typed character.
- Evidence: K/PageParts.kt:132-163 (text `"+ $title"` at :158), wrapped in `cardSurface()` at K/BuilderScreen.kt:195. The `addDashOutline` token is used only once, at K/PlayerRecordParts.kt:230.

**A19. The Library add button is a bare plus sign**
- Surface: Library header. Android A/frame-library.png, A/library.png; iOS I/LibrarySnapshotTests/testLibraryScreenPopulated.1.png.
- Rule: T28, "the round "+" that adds to a page" stays beside the title as our own drawing.
- On screen: a thin "+" character in the corner; iOS a black disc with a white plus.
- Evidence: K/LibraryScreen.kt:126 (`BasicText("+", ...)`); `ic_plus` exists but is not used here.

**A20. Library cards drop the metadata line and mastery ring**
- Surface: Library. Android A/library.png; iOS I/LibrarySnapshotTests/testLibraryScreenPopulated.1.png, testLibraryItemCards.1.png.
- Rule: Interaction principle C, "Glanceable ... Key, tempo and composer are what you read at a glance from a music stand" (T26, docs/design-principles.md:971).
- On screen: Android cards show title and composer only. iOS adds key and tempo, the linked exercise count pill, the mastery ring, and "Add composer, key and tempo" on a bare card.
- Evidence: K/LibraryScreen.kt:131-149.

**A21. Library empty state is one line in the corner**
- Surface: Library with no items. UNCOVERED on Android; iOS I/RootShellSnapshotTests/testRootShell.1.png.
- Rule: ios-ui.md:44 lists `PlaceholderContent` as the primitive; Interaction principle A, "One primary action per screen."
- On screen: Android shows "Pieces and exercises will live here." top left with nothing to tap. iOS centres a books glyph, that line, a yellow "Add piece or exercise" button and "Scan a page".
- Evidence: K/LibraryScreen.kt:83-88.

**A22. Add to session is a tick list, not library cards**
- Surface: Add to session. Android A/builder-add.png; iOS I/SessionBuilderSnapshotTests/testAddToSessionSheet.1.png.
- Rule: ios-ui.md:49, "If a primitive almost fits, add a parameter to it (as `SegmentedPills` and `LibraryItemCard` do)."
- On screen: iOS reuses the library cards (type bar, key and tempo, black outline when chosen, a plus circle when not). Android is one card of tick rows with no type bar.
- Evidence: K/AddToSessionScreen.kt:195-215 (`TickRow` inside `cardSurface()`).

**A23. Text field hints look like answers**
- Surface: item form, entry settings aim, reflection note, summary note, tags. Android A/item-form-add.png, A/builder-entry-settings.png; iOS I/LibraryFormSnapshotTests/testLibraryAddScreen.1.png.
- Rule: T35, "Values, and anything the musician reads or sets, are 17 point in full ink"; the label is `inkSecondary`, so a hint must sit below both.
- On screen: "Required", "e.g. Allegro", "Add a tag" are the same grey as the field labels and nearly as dark as a typed value; iOS hints are a pale grey.
- Evidence: K/ItemFormParts.kt:79 (placeholder `inkSecondary`).

**A24. The reflection tempo is small and its buttons are loose**
- Surface: Reflection. Android A/player-reflection.png; iOS I/SessionSummarySnapshotTests/testReflectionSheet.1.png.
- Rule: T35, "the tempo readouts ... are the only figures sized by the dial or readout they sit in."
- On screen: iOS centres "♩ = 96" large between two boxed minus and plus buttons. Android puts "♩ = 72" at body size on the left with two bare glyphs (the A8 stepper).
- Evidence: K/ReflectionSheet.kt:402 calls `StepperRow`, which sets the value at body size (K/SettingControls.kt:89-101).

**A25. Sheets open as full pages with large titles**
- Surface: entry settings, add to session, metronome, keys, variations, section, link sections. Android A/builder-entry-settings.png, A/player-click-sheet.png; iOS I/SessionBuilderSnapshotTests/testEntrySettingsSheetPopulated.1.png.
- Rule: Interaction principle C, "One concept per screen ... usually means a second screen (or a sheet)"; "every list to detail behaves the same."
- On screen: iOS sheets carry a small centred title; Android shows a 30sp page title with a divider, so a quick setting reads as a new destination. Worth a ruling rather than a fix: a full-screen dialog may be the better Android idiom.
- Evidence: K/ItemFormExercises.kt:290-291 (`FullScreenDialog`), used at K/PieceKeys.kt:161 and K/PieceVariations.kt:147; entry settings and add to session are routes built on `ScreenScaffold`.

**A26. The empty day card lacks its moon**
- Surface: Practice, a day with no practice. Android A/frame-practice.png; iOS I/FirstRunSnapshotTests/testPracticeScreenStartHere.1.png.
- Rule: Interaction principle C, "Glanceable".
- On screen: iOS shows a moon glyph above "No practice logged"; Android the words only.
- Evidence: K/PracticeParts.kt:263-297.

#### Low

**A27. The session card hand-rolls the card surface**
- Surface: Practice. Android A/practice-filled.png; iOS I/PracticeSnapshotTests/testSessionCardsPlayedSummary.1.png.
- Rule: ios-ui.md:49, hand-rolled copies are the top source of drift.
- On screen: nothing today; it matches `cardSurface()` by hand.
- Evidence: K/PracticeParts.kt:226-229 repeats K/components/CardSurface.kt:13-16.

**A28. The field card is hand-rolled inside the form**
- Surface: item form. Android A/item-form-add.png.
- Rule: T35, "A field's label sits inside its card, above the value (`FieldCard`, `FieldLabel`)".
- On screen: field cards on the form use 12dp vertical padding where `FieldCard` uses 10, so form rows sit slightly taller than other field cards.
- Evidence: K/ItemFormParts.kt:46-51 against K/components/FieldCard.kt:36.

**A29. Spacing literals where a token exists**
- Surface: Progress mastery rows and toast, progress empty state, tag chips.
- Rule: android-quality.md:62, "Every colour, font, spacing and radius comes from `ui/Theme.kt`".
- On screen: nothing visible today.
- Evidence: K/ProgressParts.kt:300 (`16.dp` is `IntradaSpacing.card`), :252-253 and :301 (14, 11, 13); K/ProgressScreen.kt:147 (`padding(32.dp)`); K/components/TagChip.kt:64-72.

**A30. Two clickables carry no role**
- Surface: Library rows, Practice session card.
- Rule: android-quality.md:81, "TalkBack labels on every control ... that read as the iOS VoiceOver label does."
- On screen (TalkBack): a Library card is read with its label but not as a button. The session card sets the role in its semantics, so only the Library row is affected.
- Evidence: K/LibraryScreen.kt:97-103 (`clickable(onClickLabel = ...)` without `role`).

### (c) Refinement observations

These are not breaches; they are places that look flat or weak and would repay a design pass.

1. **Whitespace without hierarchy on Practice.** The week strip's day numbers and letters are all the same small size and weight; today's black disc is the only accent, and the "Today / Rest day" label pair floats loosely above the dashed card (A/frame-practice.png).
2. **The player's lower half is empty.** With no aim, notes or chips, the gap between the timer ring and the rep buttons is a third of the screen (A/player.png); the timer ring never shows progress when there is no planned time, so it reads as decoration.
3. **Focus chips are oversized.** In entry settings the focus and "where" chips draw their full 48dp touch area as fill, so "Tempo" and "Whole piece" look like buttons rather than options (A/builder-entry-settings.png). iOS keeps the drawn chip smaller than its hit area.
4. **Remove crosses are heavy.** The builder's row cross and the key picker's clear cross are large bare glyphs in `inkFaintIcon`, louder than the row titles they sit beside (A/builder.png, A/item-form-key-wheel.png); iOS uses a smaller cross or a filled clear circle.
5. **Grouped block header is muddled.** The group header in the builder shows "Run" on its own line under the meta with no label, and the beige group fill against white child rows makes the block read as two unrelated cards (A/builder.png).
6. **Meta lines are one step too big.** "Piece · 8m · 66 bpm" on Session detail and "Piece" under builder rows use 15sp where iOS uses 13sp, so the title and its meta compete (A/session-detail.png, A/builder.png).
7. **The sparkle on the mastery toast is tiny.** A 15dp glyph in a 34dp butter disc reads as a speck (A/progress.png).
8. **The welcome's glyphs are light.** The three feature icons on the welcome are thinner and smaller than the iOS ones and sit off the text's first line (A/first-run-welcome.png).
9. **Practice defaults dividers stop short on the left only**, which with the square segmented track makes the card look unfinished (A/profile.png).

## Android native behaviour audit (issue #2599)

Read only review of `main` at e3cd2990. Nothing was built or run; every "today" claim comes from source or a committed Roborazzi snapshot. Paths are under `android/app/src/main/kotlin/com/intrada/android/` unless they start otherwise. Snapshots are `android/app/src/test/snapshots/<name>.png`.

The split used throughout:

- **Look** (keep intrada's): paper, ink, one highlighter, Hanken Grotesk and DM Mono, 3dp cards, the five sizes of T35. Nothing below proposes Material colour, shape or type.
- **Behaviour and structure** (Android wins): what back does, where navigation and actions sit, how modals arrive and leave, what a press feels like, how the keyboard moves you on, what undo looks like.

Effort: small is under a day in one or two files; medium is a shared primitive plus its call sites; large touches the core or adds a platform surface.


### 1. System back and predictive back

| | |
|-|-|
| **Today** | Seven `BackHandler`s. `ui/ReflectionSheet.kt:217` is `BackHandler {}`: back does nothing at all after an item. `ui/FeedbackSheet.kt:67` does nothing once a note is typed (no prompt, no feedback). `ui/BuilderScreen.kt:124`, `ui/PlayerScreen.kt:128` and `ui/SummaryScreen.kt:46` are always enabled and open a confirm or options dialog. `ui/ProfileEditSheet.kt:146` and `ui/FirstRunScreen.kt:77` step back inside the flow. Manifest has no `enableOnBackInvokedCallback`, so predictive back is on by default (`targetSdk = 36`, `app/build.gradle.kts:20`). Navigation Compose 2.10.2 is in use, which animates predictive back between destinations. |
| **Android expects** | developer.android.com, "Add support for the predictive back gesture": an enabled callback stops the predictive animation, so a callback must be enabled only while the UI state needs it ("tie enabled state to UI state"). Back must always do something visible; a back that silently does nothing reads as a frozen app. The Android quality rule (`.claude/rules/android-quality.md:87`) says the same: intercept only for unsaved input. |
| **Recommendation** | **Adopt.** Behaviour. Give every handler an `enabled` tied to real state: Builder only when the setlist has entries (it already branches on that inside), Feedback only when the note is non empty, and then show the same discard dialog iOS uses rather than swallowing. Reflection: back should save what is there and advance, or show a one line "Keep or discard your answers?" dialog; never a dead key. Player: keep the options dialog (a live session is a reason a musician recognises) but use `PredictiveBackHandler` so the gesture previews the dialog rather than doing nothing until release. |
| **Effort** | Small for the `enabled` flags and Feedback; medium for Reflection (needs a decision on what back means after an item, probably a short T ruling). |

### 2. Top app bar, up navigation and where actions sit

| | |
|-|-|
| **Today** | One header for every screen, `ui/ScreenScaffold.kt:42-69`: a 30sp title left, actions as text right, a hairline under it. No back or up arrow on any pushed screen: `piece.png` shows "Clair de Lune" with only "Edit" top right, where iOS (`ios/IntradaTests/__Snapshots__/LibraryDetailSnapshotTests/testPieceDetailWithSections.1.png`) has the back chevron. Forms put "Cancel" and "Add" side by side at the trailing edge (`item-form-add.png`, `ui/ItemFormScreen.kt:456`, `piece-keys-sheet.png`), the iOS sheet pattern T28 describes. The Metronome sheet already uses a close X (`player-click-sheet.png`), so the app disagrees with itself. Header text actions are 48dp minimum (`ui/PageParts.kt:73`). |
| **Android expects** | Material 3 "Top app bar" guidelines: a navigation icon at the leading edge (back arrow on a pushed screen, close X on a full screen dialog), the title next to it, and up to three trailing actions. M3 "Dialogs, full screen": close X leading, one confirming text action trailing ("Save", "Add"), no Cancel button, because the X is cancel. Gesture back exists, but users still look for the arrow, and TalkBack users rely on it. |
| **Recommendation** | **Adapt.** Structure from Android, look from intrada. Add an optional leading navigation slot to `ScreenScaffold`: a back arrow drawn in ink on pushed screens, an X on modal forms. Drop "Cancel" from form headers so they read X, title, Add. Keep T28's large title, the hairline and the "+" beside the Library title; keep header actions as ink text, not Material icons. T28 itself stays an iOS ruling; Android gets its own line in T36. |
| **Effort** | Medium: one primitive, about fifteen call sites (`TextAction("Cancel"` appears in eleven files). |

### 3. Bottom navigation and tabs

| | |
|-|-|
| **Today** | `ui/AppFrame.kt:253-321`: four tabs, 56dp minimum height, icon plus 13sp label, selected state shown only by ink versus secondary ink. `indication = null` at `AppFrame.kt:291`, so a tap on a tab gives no press feedback. Tab switching saves and restores state (`AppFrame.kt:240-246`), which is correct. Routines is an empty placeholder (`AppFrame.kt:135`) while iOS has `RoutinesScreen.swift`; that is port progress, not a platform question, but a dead tab is noticeable. |
| **Android expects** | Material 3 "Navigation bar": 80dp tall, an active indicator (a pill behind the selected icon), labels always shown, press feedback on each destination, reselecting the current tab scrolls to top or pops to its root. |
| **Recommendation** | **Adapt.** Keep the paper bar, ink icons and Hanken labels. Add an active indicator drawn as a highlighter pill behind the selected icon (that is the one bright colour doing its T24 job, as a fill under ink), give tabs a press indication, and pop to root on reselect. Height can rise toward 64 to 80dp; that is a look call for Jon. |
| **Effort** | Small. |

### 4. Sheets, dialogs and full screen pickers

| | |
|-|-|
| **Today** | No bottom sheets at all. Every modal is either a centred `Dialog` (`ui/PageParts.kt:378` confirm, `ui/PlayerScreen.kt:364` options, `ui/BuilderScreen.kt:253`, `ui/PieceVariations.kt:304` rename) or a `FullScreenDialog` wrapper (`ui/ItemFormExercises.kt:290`) used for Keys, Variations, the Metronome (`ui/ClickSheet.kt:53`) and exercise pickers. These open and close with no motion and sit outside Navigation, so predictive back shows nothing. `ConfirmDialog` already puts actions bottom right, Cancel then a red verb, which is Android shaped. |
| **Android expects** | Material 3 "Bottom sheets" for a short set of options or a quick setting attached to the current screen (drag handle, swipe down to dismiss, scrim, predictive back shrinks it). M3 "Dialogs" for a decision. M3 "Full screen dialogs" for a multi field task, entering from the bottom. |
| **Recommendation** | **Adapt.** Build an intrada bottom sheet primitive on Compose foundation (paper surface, 3dp top corners, ink drag handle; no Material import). Move Player options, the Metronome, the variation picker and the trouble spot sheet to it, since iOS presents them as sheets with detents too. Keep `ConfirmDialog` as is. Keep Keys and Variations full screen, but give them the X leading pattern from area 2 and an enter from bottom transition. |
| **Effort** | Medium for the primitive; small per call site after. |

### 5. Destructive actions: confirm versus undo snackbar

| | |
|-|-|
| **Today** | No snackbar or undo surface anywhere. Confirm dialogs for piece delete (`ui/PieceScreen.kt:181`, "This can't be undone."), section removal (`ui/SectionScreen.kt:107`), variation removal (`ui/VariationRow.kt:97`), photo removal (`ui/PhotoCard.kt:163`), discard session (`ui/SummaryScreen.kt:78`, `ui/BuilderScreen.kt:302`). This mirrors iOS, which uses `.alert` in the same places. The only undo is the rep undo text action (`ui/PlayerParts.kt:290`). |
| **Android expects** | Material 3 "Snackbar": a brief message at the bottom with one action, typically Undo, for a reversible removal; a dialog only when the loss cannot be undone. This is also intrada's own T6 ("recoverable actions: optimistic plus undo toast"), still unfinished on both shells. |
| **Recommendation** | **Adopt**, and it is not Android only. An intrada snackbar (ink surface, paper text, highlighter Undo, a fit with T24's dark celebration toast) for removing a section, variation, photo or setlist entry. Keep the confirm for deleting a whole piece and for discarding a played session until the core can restore them. The undo needs a core event (restore the removed thing), so this is a core change both shells share. |
| **Effort** | Large overall (core restore events, both shells); the snackbar primitive itself is small. |

### 6. Touch feedback (ripple)

| | |
|-|-|
| **Today** | No `LocalIndication` is provided anywhere and no ripple exists. Every `clickable` therefore falls back to Compose foundation's built in default press overlay, a flat translucent grey rectangle that ignores the card's 3dp corners; tabs have none at all (`AppFrame.kt:291`). detekt bans `androidx.compose.material.*` (`android/config/detekt.yml:28`), which also catches the standalone `material-ripple` artifact. |
| **Android expects** | Material 3 "States" (pressed state layer) and the platform norm since Android 5: a ripple from the touch point, clipped to the control's shape. Its absence is one of the first things that makes an app feel like a port. |
| **Recommendation** | **Adopt the behaviour, keep the look.** Provide one app wide indication: a ripple in ink at low alpha (about 10 to 12 percent), bounded and clipped to the shape it sits on, and on the highlighter button an ink ripple over butter. Either allow `androidx.compose.material.ripple` as a named exception in detekt (it carries no Material colour or component) or write an `IndicationNodeFactory` in `ui/`. The rule in `android-quality.md` needs one line saying which. |
| **Effort** | Small to medium (one provider; a sweep for any `clickable` that should stay quiet, such as the player surface). |

### 7. Edge to edge and the system bars

| | |
|-|-|
| **Today** | `MainActivity.kt:83` calls `enableEdgeToEdge()` with no arguments. Screens pad `WindowInsets.safeDrawing` (`ui/ScreenScaffold.kt:40`, `PlayerScreen.kt:134`, `FirstRunScreen.kt:127`); the tab bar pads the navigation bar and cutout (`AppFrame.kt:258-262`). The manifest theme is `@android:style/Theme.Material.Light.NoActionBar` and there is no `res/values` folder, so no `windowBackground`, no splash theme. |
| **Android expects** | developer.android.com "Display content edge to edge": with no arguments, `enableEdgeToEdge` picks status bar icon colour from the system's dark setting, so on a phone in dark mode a light only app gets light icons on paper, close to invisible. Android 12 and later always show a system splash (icon on `windowBackground`, "Splash screens" guide). |
| **Recommendation** | **Adopt.** Pass `SystemBarStyle.light(paper, paper)` for both bars until dark exists. Set `windowBackground` to paper in a small theme so the launch frame and the system splash are paper, not white, and use `core-splashscreen` so the system splash hands over to T34's wordmark splash rather than stacking a second one. Look stays intrada's. |
| **Effort** | Small. |

### 8. Text fields, keyboard and IME actions

| | |
|-|-|
| **Today** | `ui/ItemFormParts.kt:72` sets capitalisation and keyboard type but no `imeAction` or `KeyboardActions`, so the keyboard shows a generic Enter on every single line field. Only the picker search sets `ImeAction.Search` (`ui/AddToSessionScreen.kt:236`). `safeDrawing` includes the IME inset, so forms do move above the keyboard. Fields are `BasicTextField` with intrada decoration, which is right. |
| **Android expects** | developer.android.com "Handle input method actions" and Material 3 "Text fields": Next moves to the following field, Done on the last closes the keyboard (or submits), Search on search fields; `keyboardType = Number` for figures such as bpm. |
| **Recommendation** | **Adopt.** Behaviour only. `imeAction = Next` with `focusManager.moveFocus(Down)` through Title, Composer, Tempo marking, Beats per minute; `Done` on the last single line field; a tag field's Done adds the tag. Check bpm and bar fields use the number keyboard. |
| **Effort** | Small. |

### 9. Date, time, number and tempo pickers

| | |
|-|-|
| **Today** | No date or time picker exists on either shell. Numbers use steppers: Beats in the bar shows a value with minus and plus (`player-click-sheet.png`), `ui/EntryFocusCard.kt:80` `TargetStepper`. Keys use intrada's own circle of fifths (`ui/KeyWheel.kt`, `piece-keys-sheet.png`). Time signatures and groupings are segmented pills. |
| **Android expects** | Material 3 "Date pickers" and "Time pickers" when those arrive; for numbers, either a stepper or a text field with the number keyboard is normal. There is no Android wheel picker convention to copy. |
| **Recommendation** | **Keep intrada's.** The key wheel and steppers are domain controls, not platform chrome. Two adaptations: let a tap on the stepper's value open a number keyboard for a big jump (bpm 60 to 132 should not be 72 taps), and give the wheel and steppers the area 6 ripple and area 11 haptics. If a date picker is ever needed, use the M3 behaviour (modal, calendar grid, text entry toggle) in intrada's look. |
| **Effort** | Small. |

### 10. Swipe actions on rows

| | |
|-|-|
| **Today** | None. iOS has swipe to remove on builder rows (`ios/Intrada/Views/Screens/SessionBuilderScreen.swift:657`). Android reaches the same action through Edit mode's leading remove button (`ui/BuilderRowViews.kt:120`), which satisfies T4's floor. |
| **Android expects** | Material 3 "Lists" allows swipe to dismiss as an accelerator, usually paired with an undo snackbar; it is far less expected on Android than on iOS, and many first party apps skip it. |
| **Recommendation** | **Adapt, later.** Add swipe to remove on builder rows only once the undo snackbar (area 5) exists, so a stray swipe is recoverable. Not worth doing before. |
| **Effort** | Medium (after area 5). |

### 11. Long press and context menus

| | |
|-|-|
| **Today** | One long press: rename on a variation row (`ui/PieceVariations.kt:291`, through `TickRow` at `ui/PageParts.kt:245`), with a TalkBack custom action, which is good. The Library row has no long press, where iOS has a context menu to add or remove from priorities (`ios/Intrada/Views/Screens/LibraryScreen.swift:100`). |
| **Android expects** | Material 3 "Menus": long press on a list item opens a dropdown menu anchored to the item (not a centred dialog), with a haptic on open; on lists that support it, long press can also start multi select. TalkBack exposes the same actions as custom actions. |
| **Recommendation** | **Adapt.** An intrada anchored menu primitive (paper card, 3dp corners, ink text rows, 48dp each) on long press, starting with the Library priority toggle to reach parity with iOS. Keep every action reachable elsewhere (T4). |
| **Effort** | Medium (new primitive), then small per site. |

### 12. Haptics

| | |
|-|-|
| **Today** | `ui/Haptic.kt` has two kinds, `SELECTION` (`SegmentTick`) and `IMPACT` (`LongPress`), used only in the key picker (`PieceKeys.kt`, `KeyWheel.kt`, `KeyWords.kt`). iOS has haptics in about thirty files, including success after a save (`Store+Feedback.swift`), score and tick rows, steppers and the player. T30 says a successful tap "keeps its success haptic". |
| **Android expects** | developer.android.com "Haptics design principles" and `HapticFeedbackConstants`: `CONFIRM` and `REJECT` for outcomes (API 30 and up), `SEGMENT_TICK` or `CLOCK_TICK` for steps, `LONG_PRESS` for long press, and nothing on ordinary taps. The system touch feedback setting is respected when calls go through `performHapticFeedback`. |
| **Recommendation** | **Adapt.** Keep one gateway (`Haptic.kt`) and add `SUCCESS` (`Confirm`) and `FAILURE` (`Reject`), falling back to nothing below API 30 (minSdk is 28). Fire them where iOS does: a confirmed save, Got it and Not quite, stepper steps, long press menus. Android users expect fewer and lighter haptics than iOS; do not copy every iOS site, only outcomes and steps. Needs a real device (`specs/android-shell.md:120-123`). |
| **Effort** | Small to medium. |

### 13. Typography against Android's defaults

| | |
|-|-|
| **Today** | `ui/Theme.kt:179-198`: T35's five sizes in sp (30, 20, 17, 15, 13), Hanken and DM Mono. Tab label 13sp. dp and pt are physically close, so sizes match iOS on screen. |
| **Android expects** | Material 3 "Type scale" is a default, not a requirement: body large 16sp, title large 22sp, label large 14sp, navigation bar label 12sp. Custom brand type is normal on Android. |
| **Recommendation** | **Keep intrada's.** 17sp body is one step above Android's 16 and reads fine; T26's reasoning (arm's length from a music stand) holds on any phone. No change. |
| **Effort** | None. |

### 14. System font scaling

| | |
|-|-|
| **Today** | All text is in sp, so it scales. Icons and dots scale with `fontScale` up to a cap (`ui/components/IconScale.kt:12`, `StartHereCard.kt:139`, `ClickBarLine.kt:161`, `PracticeParts.kt:181`). Snapshots exist only at the default scale; large scale ones are deferred (#2134). |
| **Android expects** | Android 14 "Non linear font scaling" up to 200 percent: large text grows less than small text, and layouts must not clip. developer.android.com "Support different font sizes" asks for testing at the largest scale. |
| **Recommendation** | **Adopt** the testing, keep the look. Bring #2134 forward for the five busiest screens (Library, Piece, Item form, Player, Metronome) at 200 percent; the 30sp title and the player timer are where clipping will show first. |
| **Effort** | Medium (snapshots plus whatever they find). |

### 15. Share sheet, notifications, widgets, background audio

| | |
|-|-|
| **Today** | None on either shell. The metronome stops a grace period after the app leaves the foreground (`ui/ClickController.kt:149-151`); the player keeps the screen on (`PlayerScreen.kt:92`). |
| **Android expects** | A metronome that keeps playing with the screen off runs as a foreground service with a media style notification (developer.android.com "Foreground service types: mediaPlayback"); Android 13 needs the notification permission asked at the moment it matters. Widgets (Glance) and the share sheet (`ACTION_SEND`) are optional. |
| **Recommendation** | **Keep for now; note for later.** Matches iOS today. If the click should survive the screen locking, that is a foreground service plus a notification with Stop, in intrada's words; worth a spec of its own, not this audit. |
| **Effort** | Large, if ever. |

### 16. App icon (adaptive and themed)

| | |
|-|-|
| **Today** | `res/mipmap-anydpi/ic_launcher.xml` is an adaptive icon with background, foreground and a `monochrome` layer, so Android 13 themed icons work. The foreground (`res/drawable/ic_launcher_foreground.xml`) sits well inside the 66dp safe zone. It still fills with the old brown ink `#3B2A1E`, which T27 replaced with `#2A2725`. |
| **Android expects** | developer.android.com "Adaptive icons": all three layers, content inside the safe zone. Met. |
| **Recommendation** | **Keep.** Only update the foreground ink to the current `ink` so the icon matches the app. |
| **Effort** | Small. |

### 17. Dark theme and dynamic colour

| | |
|-|-|
| **Today** | Light only. T24 parks dark (#1679 for the player). No `values-night`. |
| **Android expects** | Material 3 "Dynamic colour" is optional and many branded apps skip it; a dark theme is expected by many Android users, but a light only app is acceptable if it does not break under system dark. |
| **Recommendation** | **Keep iOS style (parked), with one guard:** the system bars fix in area 7, so system dark cannot make the status bar unreadable. **Do not adopt dynamic colour**: wallpaper colours would replace the highlighter, which is the brand. When dark ships, it ships as intrada's dark, not Material You. |
| **Effort** | Small for the guard; dark itself is out of scope. |


### Draft principle

#### T36: Android behaves like Android, and looks like intrada

**Status:** PROPOSED (jonyardley/intrada#2599). On Android, intrada keeps its look and adopts the platform's behaviour. The look is paper, ink, one highlighter, Hanken Grotesk and DM Mono, the five sizes and the 3dp card; no Material colour, shape or type stands in for it, and wallpaper colour never replaces the highlighter. Behaviour and structure follow Android: back always does something, and predictive back shows where it goes; a pushed screen has a back arrow and a modal form has a close cross, with its one action on the right; a press ripples in ink; quick choices rise as bottom sheets; a reversible removal offers Undo rather than asking first; the keyboard's Next and Done move you on. When an iOS ruling such as T28 is about where something sits or how it moves, it binds iOS only, and Android takes the platform's answer drawn in intrada's look.


### The five changes that would most make it feel native

1. **Back that always works and previews** (area 1). Remove the dead `BackHandler {}` on reflection, give Feedback a discard prompt, and tie every handler's `enabled` to real unsaved state so predictive back animates. Small; a broken back is the fastest way to feel foreign.
2. **A navigation icon in the header** (area 2). Back arrow on pushed screens, close X with one trailing action on modal forms, Cancel dropped. Medium; changes how every pushed screen and form reads.
3. **Ink ripple on every press** (area 6). One app wide indication, clipped to shape, plus a ruling on the detekt exception. Small to medium; felt on every tap.
4. **Bottom sheets for quick settings** (area 4). An intrada sheet primitive for player options, the Metronome and the variation picker, with drag to dismiss and predictive back. Medium.
5. **Undo instead of asking, for recoverable removals** (area 5). An intrada snackbar with Undo for sections, variations, photos and setlist entries, backed by core restore events both shells share; finishes T6 too. Large, so it can follow the four above.

Close behind: the system bar style and paper launch frame (area 7, small and worth doing first because it is a visible bug under system dark), IME actions on forms (area 8), and the tab bar's active indicator (area 3).
