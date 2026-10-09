package com.intrada.android

import com.intrada.android.core.LiveBridge
import com.intrada.android.core.withIds
import com.intrada.ffi.clickTempoWords
import com.intrada.shared.Accidental
import com.intrada.shared.AppEffect
import com.intrada.shared.BarRange
import com.intrada.shared.BarsInput
import com.intrada.shared.ClickBarOption
import com.intrada.shared.ClickPreset
import com.intrada.shared.ClickPresetOption
import com.intrada.shared.CreateItem
import com.intrada.shared.Effect
import com.intrada.shared.Event
import com.intrada.shared.ExerciseLink
import com.intrada.shared.Felt
import com.intrada.shared.FocusKind
import com.intrada.shared.FocusTargetView
import com.intrada.shared.IntentionFocus
import com.intrada.shared.Item
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKind
import com.intrada.shared.ItemSection
import com.intrada.shared.Key
import com.intrada.shared.Letter
import com.intrada.shared.LibraryItemView
import com.intrada.shared.LinkChange
import com.intrada.shared.LinkTarget
import com.intrada.shared.LinkedSectionView
import com.intrada.shared.Modality
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.PersistenceOutput
import com.intrada.shared.Request
import com.intrada.shared.SectionChange
import com.intrada.shared.SectionEdit
import com.intrada.shared.SectionKind
import com.intrada.shared.Segment
import com.intrada.shared.SessionEvent
import com.intrada.shared.TempoBand
import com.intrada.shared.TempoInput
import com.intrada.shared.TempoReading
import com.intrada.shared.Variation
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

// Through the host build of the real core, never a stub: a stub cannot catch a decode that skews
// from the Rust side (#846).
class BridgeRoundTripTest {
    @Test
    fun itemsEncodedInKotlinComeBackAsLibraryRows() {
        val bridge = LiveBridge()
        val load =
            bridge.update(Event.StartApp).single {
                (it.effect as? Effect.Persistence)?.value == PersistenceOperation.LoadItems
            }

        val requests = bridge.resolve(load.id, PersistenceOutput.Items(Fixtures.library))

        val rows = libraryChanged(requests)
        val view = bridge.view()
        val shown = rows.withIds(view.visibleIds)
        assertEquals(
            setOf("Clair de Lune", "Gymnopédie No. 1", "Hanon No. 1"),
            shown.map { it.title }.toSet(),
        )
        val satie = shown.single { it.title == "Gymnopédie No. 1" }
        assertEquals("Erik Satie", satie.subtitle)
        assertEquals(Key(Letter.D, Accidental.NATURAL, Modality.MAJOR), satie.key)
        assertEquals("D major", satie.keyLabel)
        assertEquals("Lent", satie.tempoMarking)
        assertEquals(70.toUShort(), satie.tempoBpm)
        assertEquals("Lent · ♩ = 70", satie.tempoLine)
        assertEquals("Lent, 70 beats per minute", satie.tempoLineSpoken)
        assertEquals(listOf("recital"), satie.tags)
        assertEquals("2 pieces · 1 exercise", view.libraryCountLine)
    }

    @Test
    fun theCoreSampleSetDecodes() {
        val bridge = LiveBridge()

        val rows = libraryChanged(bridge.update(Event.LoadSampleData))

        val view = bridge.view()
        assertTrue(rows.any { it.title == "Clair de Lune" && it.subtitle == "Claude Debussy" })
        assertEquals(rows.size, view.visibleIds.size)
    }

    // A second decoder of the same wire: a skewed TempoInput shows only here (#846, #2224).
    @Test
    fun aTypedBpmCrossesAsTextAndLandsAsANumber() {
        val bridge = LiveBridge()

        val saved =
            bridge
                .update(
                    Event.Item(
                        ItemEvent.Add(
                            CreateItem(
                                title = "Nocturne",
                                kind = ItemKind.PIECE,
                                tempo = TempoInput(marking = "Lento", bpm = " 60 "),
                                tags = emptyList(),
                                variationLabels = emptyList(),
                            )
                        )
                    )
                )
                .mapNotNull {
                    ((it.effect as? Effect.Persistence)?.value as? PersistenceOperation.SaveItem)
                        ?.value
                }
                .single()

        assertEquals("Lento", saved.tempo?.marking)
        assertEquals(60.toUShort(), saved.tempo?.bpm)
    }

    // Sections cross on a second decoder: a skewed BarsInput or SectionView shows only here (#846,
    // #2245).
    @Test
    fun sectionsCrossTheBridgeTypedAndPicked() {
        val bridge = LiveBridge()
        val added =
            bridge.update(
                Event.Item(
                    ItemEvent.Add(
                        CreateItem(
                            title = "Rondo",
                            kind = ItemKind.PIECE,
                            tags = emptyList(),
                            variationLabels = emptyList(),
                        )
                    )
                )
            )
        val id =
            added
                .mapNotNull {
                    ((it.effect as? Effect.Persistence)?.value as? PersistenceOperation.SaveItem)
                        ?.value
                }
                .single()
                .id

        val saved =
            addSections(
                bridge,
                id,
                listOf(
                    SectionEdit(
                        null,
                        "A1",
                        BarsInput.Typed("1-16"),
                        SectionKind.FORM,
                        "",
                    ),
                    SectionEdit(
                        null,
                        "",
                        BarsInput.Picked(12.toUShort(), 14.toUShort()),
                        SectionKind.TROUBLESPOT,
                        "72",
                    ),
                ),
            )

        val sections: List<ItemSection> = saved.sections
        assertEquals(
            listOf(BarRange(1.toUShort(), 16.toUShort()), BarRange(12.toUShort(), 14.toUShort())),
            sections.map { it.bars },
        )
        assertEquals(72.toUShort(), sections.last().targetBpm)
    }

    @Test
    fun theClicksTempoWordsComeFromTheCore() {
        val words = clickTempoWords(168u, 8u)
        assertEquals("♪ = 168", words.text)
        assertEquals("168 quaver beats per minute", words.spoken)
    }

    // The click's band and bars sit mid-ViewModel, so a skew here also garbles the fields after
    // them (#2225).
    @Test
    fun theClicksBandAndBarsDecode() {
        val limits = LiveBridge().view().limits

        assertEquals(
            TempoBand(unit = 8.toUByte(), min = 80.toUShort(), max = 416.toUShort()),
            limits.clickTempoBands.single { it.unit == 8.toUByte() },
        )
        val groups = listOf(3, 2, 2).map(Int::toUByte)
        assertEquals(
            ClickBarOption(
                beats = 7.toUByte(),
                groups = groups,
                presets =
                    listOf(
                        ClickPresetOption(ClickPreset.EVERYBEAT, 0b1111111.toUShort()),
                        ClickPresetOption(ClickPreset.GROUPSTARTS, 0b0101001.toUShort()),
                        ClickPresetOption(ClickPreset.DOWNBEAT, 1.toUShort()),
                    ),
            ),
            limits.clickBars.single { it.beats == 7.toUByte() && it.groups == groups },
        )
    }

    // The sheet's stop moment crosses last in the reflection view (#2297).
    @Test
    fun theSheetsStopMomentDecodes() {
        val bridge = LiveBridge()
        val load =
            bridge.update(Event.StartApp).single {
                (it.effect as? Effect.Persistence)?.value == PersistenceOperation.LoadItems
            }
        val rows =
            libraryChanged(bridge.resolve(load.id, PersistenceOutput.Items(Fixtures.library)))
        bridge.update(Event.Session(SessionEvent.StartBuilding))
        bridge.update(Event.Session(SessionEvent.AddToSetlist(rows.first().id)))
        bridge.update(Event.Session(SessionEvent.StartSession("2026-09-27T09:00:00Z")))

        bridge.update(
            Event.Session(
                SessionEvent.PrepareReflection(
                    "2026-09-27T09:05:00Z",
                    TempoReading(bpm = 72.toUShort(), clickSounding = false),
                )
            )
        )

        assertEquals(
            "2026-09-27T09:05:00+00:00",
            bridge.view().activeSession?.reflection?.stoppedAt,
        )
        assertEquals(300uL, bridge.view().activeSession?.reflection?.elapsedSecs)
    }

    // A play names a key and two variations on Kotlin's own encoder, and the new library-wide
    // variations, a tombstone among them, load on Kotlin's encoder (#846, #2246).
    @Test
    fun aPlayInAKeyWithTwoVariationsDecodes() {
        val bridge = LiveBridge()
        val loads = bridge.update(Event.StartApp)
        val loadVariations = loads.single {
            (it.effect as? Effect.Persistence)?.value == PersistenceOperation.LoadVariations
        }
        bridge.resolve(
            loadVariations.id,
            PersistenceOutput.Variations(
                listOf(
                    Variation("v-swung", "Swung", "2026-10-01T09:00:00Z", null),
                    Variation("v-gone", "Gone", "2026-10-01T09:00:00Z", "2026-10-02T09:00:00Z"),
                )
            ),
        )
        assertEquals(listOf("v-swung"), bridge.view().variations.map { it.id })

        val added =
            bridge.update(
                Event.Item(
                    ItemEvent.Add(
                        CreateItem(
                            title = "Scales",
                            kind = ItemKind.EXERCISE,
                            tags = emptyList(),
                            variationLabels = listOf("Slow", "Swung"),
                        )
                    )
                )
            )
        val item =
            added
                .mapNotNull {
                    ((it.effect as? Effect.Persistence)?.value as? PersistenceOperation.SaveItem)
                        ?.value
                }
                .single()
        bridge.update(Event.Session(SessionEvent.StartBuilding))
        bridge.update(Event.Session(SessionEvent.AddToSetlist(item.id)))
        bridge.update(Event.Session(SessionEvent.StartSession("2026-10-03T09:00:00Z")))
        val entryId = bridge.view().activeSession?.entries?.firstOrNull()?.id.orEmpty()
        val eFlat = Key(Letter.E, Accidental.FLAT, Modality.MAJOR)

        bridge.update(
            Event.Session(
                SessionEvent.SwitchPlay(
                    entryId,
                    null,
                    eFlat,
                    item.variationIds.reversed(),
                    "2026-10-03T09:01:00Z",
                    TempoReading(bpm = 72.toUShort(), clickSounding = false),
                )
            )
        )

        val active = bridge.view().activeSession
        assertEquals(eFlat, active?.currentKey)
        assertEquals(item.variationIds.reversed(), active?.currentVariationIds)
        assertEquals(2, item.variationIds.size)
    }

    // An empty first load seeds the four built-ins, which leave the core on Kotlin's decoder.
    @Test
    fun theBuiltInVariationsAreSavedOnAnEmptyFirstLoad() {
        val bridge = LiveBridge()
        val loadVariations =
            bridge.update(Event.StartApp).single {
                (it.effect as? Effect.Persistence)?.value == PersistenceOperation.LoadVariations
            }

        val saved =
            bridge
                .resolve(loadVariations.id, PersistenceOutput.Variations(emptyList()))
                .mapNotNull {
                    ((it.effect as? Effect.Persistence)?.value
                            as? PersistenceOperation.SaveVariations)
                        ?.value
                }
                .single()

        assertEquals(
            listOf("Hands separately", "Dotted rhythms", "Back to front", "Three chord tones only"),
            saved.map { it.label },
        )
        assertEquals(saved.map { it.label }, bridge.view().variations.map { it.label })
    }

    // Section links cross on a second decoder: a skewed LinkTarget, LinkChange.Set or link view
    // shows only here (#846, #2248).
    @Test
    fun anExerciseLinkedToSectionsOfTwoPiecesDecodes() {
        val bridge = LiveBridge()
        val nocturne = addItem(bridge, "Nocturne", ItemKind.PIECE)
        val etude = addItem(bridge, "Étude", ItemKind.PIECE)
        val thirds = addItem(bridge, "Thirds", ItemKind.EXERCISE)
        val a2 = addSection(bridge, nocturne, "A2")
        val coda = addSection(bridge, etude, "Coda")

        val written: List<ExerciseLink> =
            bridge
                .update(
                    Event.Item(
                        ItemEvent.SetExerciseLinks(
                            thirds,
                            listOf(LinkTarget(nocturne, a2), LinkTarget(etude, coda)),
                        )
                    )
                )
                .mapNotNull {
                    ((it.effect as? Effect.Persistence)?.value as? PersistenceOperation.SaveItems)
                        ?.value
                }
                .single()
                .flatMap { it.exerciseLinks }

        assertEquals(setOf(a2, coda), written.map { it.sectionId }.toSet())
        assertEquals(listOf(thirds, thirds), written.map { it.exerciseId })

        val rows =
            libraryChanged(
                bridge.update(
                    Event.Item(
                        ItemEvent.ChangePieceLink(
                            nocturne,
                            LinkChange.Set(thirds, true, listOf(a2)),
                        )
                    )
                )
            )
        val card = rows.single { it.id == nocturne }.linkedExercises.single()
        val sections: List<LinkedSectionView> = card.sections
        assertTrue(card.wholePiece)
        assertEquals(listOf("A2"), sections.map { it.label })
        val usedIn = rows.single { it.id == thirds }.usedIn
        assertEquals(
            setOf("A2", "Coda"),
            usedIn.flatMap { row -> row.sections.map { it.label } }.toSet(),
        )
        assertEquals(listOf("A2"), sections.map { it.labelInText })
        assertEquals("For the whole piece and A2", card.linkCaption)
    }

    // Links loaded from the store cross into the core on Kotlin's encoder, a tombstone among them
    // (#846, #2248).
    @Test
    fun loadedLinksReachThePieceAndItsExercise() {
        val hanon = "01J0000000000000000000HANO"
        val piece = pieceWithLoadedLinks(hanon)
        val bridge = LiveBridge()
        val load =
            bridge.update(Event.StartApp).single {
                (it.effect as? Effect.Persistence)?.value == PersistenceOperation.LoadItems
            }

        val rows =
            libraryChanged(
                bridge.resolve(
                    load.id,
                    PersistenceOutput.Items(listOf(piece) + Fixtures.library.drop(1)),
                )
            )

        val card = rows.single { it.id == piece.id }.linkedExercises.single()
        assertEquals(hanon, card.id)
        assertTrue(card.wholePiece)
        assertEquals("the tombstone stays hidden", listOf("s-spot"), card.sections.map { it.id })
        assertEquals(listOf("Bars 19 to 20"), card.sections.map { it.label })
        assertEquals(listOf("bars 19 to 20"), card.sections.map { it.labelInText })
        assertEquals("For the whole piece and bars 19 to 20", card.linkCaption)
        assertEquals(
            listOf("", "19 to 20"),
            rows.single { it.id == piece.id }.sections.map { it.barsFieldText },
        )
        val usedIn = rows.single { it.id == hanon }.usedIn.single()
        assertTrue(usedIn.linked)
        assertEquals("For the whole piece and bars 19 to 20", usedIn.linkCaption)
    }

    // A live whole-piece link, a tombstoned one to A1 and a live one to a bars-only spot.
    private fun pieceWithLoadedLinks(hanon: String) =
        Fixtures.item(
                exerciseLinks =
                    listOf(
                        ExerciseLink("l1", hanon, null, 0uL, "2026-10-04T09:00:00Z", null),
                        ExerciseLink(
                            "l2",
                            hanon,
                            "s-a1",
                            1uL,
                            "2026-10-04T09:05:00Z",
                            "2026-10-04T09:05:00Z",
                        ),
                        ExerciseLink("l3", hanon, "s-spot", 2uL, "2026-10-04T09:05:00Z", null),
                    )
            )
            .copy(
                sections =
                    listOf(
                        ItemSection(
                            "s-a1",
                            "A1",
                            null,
                            SectionKind.FORM,
                            null,
                            0uL,
                            "2026-10-04T09:00:00Z",
                            null,
                        ),
                        ItemSection(
                            "s-spot",
                            "",
                            BarRange(19.toUShort(), 20.toUShort()),
                            SectionKind.TROUBLESPOT,
                            null,
                            1uL,
                            "2026-10-04T09:00:00Z",
                            null,
                        ),
                    )
            )

    // Segments, a focus and time away cross on Kotlin's encoder and come back on its decoder
    // (#846, #2249).
    @Test
    fun segmentsAndTimeAwayRunThroughThePracticeScreen() {
        val bridge = LiveBridge()
        bridge.update(Event.StartApp)
        val nocturne = addItem(bridge, "Nocturne", ItemKind.PIECE)
        val sections =
            addSections(
                    bridge,
                    nocturne,
                    listOf("A", "B").map {
                        SectionEdit(null, it, BarsInput.Blank, SectionKind.FORM, "")
                    },
                )
                .sections
                .map { it.id }
        bridge.update(Event.Session(SessionEvent.StartBuilding))
        bridge.update(Event.Session(SessionEvent.AddToSetlist(nocturne)))
        val entryId = bridge.view().buildingSetlist?.entries?.firstOrNull()?.id.orEmpty()
        bridge.update(Event.Session(SessionEvent.SetEntryDuration(entryId, 1200u)))
        bridge.update(
            Event.Session(SessionEvent.SetSegments(entryId, sections.map { Segment(it, 0u) }))
        )
        val focus = IntentionFocus(FocusKind.TEMPO, sections.first(), 84.toUShort())
        bridge.update(Event.Session(SessionEvent.SetFocus(entryId, focus)))
        val record = bridge.view().buildingSetlist?.entries?.firstOrNull()?.record
        assertEquals(listOf(600u, 600u), record?.segments?.map { it.plannedSecs })
        assertEquals("A at 84", record?.focus?.label)

        bridge.update(Event.Session(SessionEvent.StartSession("2026-10-04T09:00:00Z")))
        bridge.update(Event.Session(SessionEvent.WentAway("2026-10-04T09:01:00Z")))
        bridge.update(Event.Session(SessionEvent.CameBack("2026-10-04T09:07:00Z")))
        assertEquals(6u, bridge.view().activeSession?.record?.awayOffer?.minutes)
        bridge.update(Event.Session(SessionEvent.LeaveAwayOut))
        bridge.update(
            Event.Session(
                SessionEvent.MoveToNextSegment(
                    "2026-10-04T09:16:40Z",
                    TempoReading(bpm = 84.toUShort(), clickSounding = false),
                )
            )
        )

        val active = bridge.view().activeSession
        assertEquals(sections.last(), active?.currentSectionId)
        assertEquals("B", active?.record?.segment?.label)
        assertEquals(640uL, active?.entries?.firstOrNull()?.plays?.firstOrNull()?.seconds)
    }

    // The builder rules the core owns cross both ways on Kotlin's codec (#2393).
    @Test
    fun minuteStepsVariationsAndFocusTargetsCrossTheBridge() {
        val bridge = LiveBridge()
        bridge.update(Event.StartApp)
        val nocturne = addItem(bridge, "Nocturne", ItemKind.PIECE)
        bridge.update(
            Event.Item(ItemEvent.UpdateItemVariations(nocturne, emptyList(), listOf("Dotted")))
        )
        val dotted = bridge.view().variations.single { it.label == "Dotted" }.id
        val sections =
            addSections(
                    bridge,
                    nocturne,
                    listOf("A", "B").map {
                        SectionEdit(null, it, BarsInput.Blank, SectionKind.FORM, "")
                    },
                )
                .sections
                .map { it.id }
        bridge.update(Event.Session(SessionEvent.StartBuilding))
        bridge.update(Event.Session(SessionEvent.AddToSetlist(nocturne)))
        val entryId = bridge.view().buildingSetlist?.entries?.firstOrNull()?.id.orEmpty()
        bridge.update(Event.Session(SessionEvent.SetEntryDuration(entryId, 600u)))
        bridge.update(
            Event.Session(SessionEvent.SetSegments(entryId, sections.map { Segment(it, 0u) }))
        )

        bridge.update(Event.Session(SessionEvent.StepSegment(entryId, sections.first(), 1)))
        bridge.update(Event.Session(SessionEvent.SetEntryVariations(entryId, listOf(dotted))))
        bridge.update(
            Event.Session(
                SessionEvent.SetFocus(entryId, IntentionFocus(FocusKind.CLEANREPS, null, null))
            )
        )

        val building = bridge.view().buildingSetlist
        val record = building?.entries?.firstOrNull()?.record
        assertEquals(listOf(dotted), building?.entries?.firstOrNull()?.plannedVariationIds)
        assertEquals(listOf(360u, 240u), record?.segments?.map { it.plannedSecs })
        assertEquals(listOf(true, true), record?.segments?.map { it.canAddMinute })
        assertEquals("1 clean in a row", record?.focus?.targetCaption)
        assertEquals(
            listOf(FocusTargetView(40u, 208u, 2u), FocusTargetView(1u, 100u, 1u), null, null),
            building?.focusChoices?.map { it.target },
        )
    }

    // Adding and removing a section keep the tuned minutes (#2398).
    @Test
    fun addingAndRemovingASectionCrossTheBridge() {
        val bridge = LiveBridge()
        bridge.update(Event.StartApp)
        val nocturne = addItem(bridge, "Nocturne", ItemKind.PIECE)
        val sections =
            addSections(
                    bridge,
                    nocturne,
                    listOf("A", "B", "C").map {
                        SectionEdit(null, it, BarsInput.Blank, SectionKind.FORM, "")
                    },
                )
                .sections
                .map { it.id }
        bridge.update(Event.Session(SessionEvent.StartBuilding))
        bridge.update(Event.Session(SessionEvent.AddToSetlist(nocturne)))
        val entryId = bridge.view().buildingSetlist?.entries?.firstOrNull()?.id.orEmpty()
        bridge.update(Event.Session(SessionEvent.SetEntryDuration(entryId, 720u)))
        val tuned = listOf(Segment(sections[0], 420u), Segment(sections[1], 300u))
        bridge.update(Event.Session(SessionEvent.SetSegments(entryId, tuned)))

        bridge.update(Event.Session(SessionEvent.AddSegment(entryId, sections[2])))
        val added = bridge.view().buildingSetlist?.entries?.firstOrNull()?.record?.segments
        assertEquals(listOf(180u, 300u, 240u), added?.map { it.plannedSecs })
        assertEquals(
            true,
            bridge.view().buildingSetlist?.entries?.firstOrNull()?.record?.canAddSection,
        )

        bridge.update(Event.Session(SessionEvent.RemoveSegment(entryId, sections[1])))
        val removed = bridge.view().buildingSetlist?.entries?.firstOrNull()?.record?.segments
        assertEquals(listOf(sections[0], sections[2]), removed?.map { it.sectionId })
        assertEquals(listOf(180u, 540u), removed?.map { it.plannedSecs })
    }

    @Test
    fun theBuilderAndTheFinishSheetCarryTheChoiceWordsFromTheCore() {
        val bridge = LiveBridge()
        bridge.update(Event.StartApp)
        val nocturne = addItem(bridge, "Nocturne", ItemKind.PIECE)
        bridge.update(Event.Session(SessionEvent.StartBuilding))
        bridge.update(Event.Session(SessionEvent.AddToSetlist(nocturne)))
        val focus = bridge.view().buildingSetlist?.focusChoices
        assertEquals(
            listOf("Tempo", "Clean in a row", "From memory", "Evenness"),
            focus?.map { it.label },
        )
        assertEquals(
            listOf(FocusKind.TEMPO, FocusKind.CLEANREPS, FocusKind.FROMMEMORY, FocusKind.EVENNESS),
            focus?.map { it.kind },
        )

        bridge.update(Event.Session(SessionEvent.StartSession("2026-10-04T09:00:00Z")))
        bridge.update(
            Event.Session(
                SessionEvent.PrepareReflection(
                    "2026-10-04T09:05:00Z",
                    TempoReading(bpm = 84.toUShort(), clickSounding = false),
                )
            )
        )
        val felt = bridge.view().activeSession?.record?.finish?.feltChoices
        assertEquals(listOf("Comfortable", "Hard work", "Strained"), felt?.map { it.label })
        assertEquals(listOf(Felt.COMFORTABLE, Felt.HARDWORK, Felt.STRAINED), felt?.map { it.felt })
        val active = bridge.view().activeSession
        assertEquals(
            active?.entries?.first()?.plays?.map { it.id },
            active?.record?.finish?.rows?.map { it.playId },
        )
    }

    private fun addItem(
        bridge: LiveBridge,
        title: String,
        kind: ItemKind,
    ): String =
        bridge
            .update(
                Event.Item(
                    ItemEvent.Add(
                        CreateItem(
                            title = title,
                            kind = kind,
                            tags = emptyList(),
                            variationLabels = emptyList(),
                        )
                    )
                )
            )
            .mapNotNull {
                ((it.effect as? Effect.Persistence)?.value as? PersistenceOperation.SaveItem)?.value
            }
            .single()
            .id

    private fun addSection(bridge: LiveBridge, piece: String, name: String): String =
        addSections(
                bridge,
                piece,
                listOf(SectionEdit(null, name, BarsInput.Blank, SectionKind.FORM, "")),
            )
            .sections
            .single()
            .id

    // Each edit saved as a new section, one change at a time (#2447); the last save holds them
    // all.
    private fun addSections(bridge: LiveBridge, piece: String, edits: List<SectionEdit>): Item =
        edits
            .map { edit ->
                bridge
                    .update(Event.Item(ItemEvent.ChangeSection(piece, SectionChange.Save(edit))))
                    .mapNotNull {
                        ((it.effect as? Effect.Persistence)?.value
                                as? PersistenceOperation.SaveItem)
                            ?.value
                    }
                    .single()
            }
            .last()

    private fun libraryChanged(requests: List<Request>): List<LibraryItemView> =
        requests
            .mapNotNull { ((it.effect as? Effect.App)?.value as? AppEffect.LibraryChanged)?.value }
            .single()
}
