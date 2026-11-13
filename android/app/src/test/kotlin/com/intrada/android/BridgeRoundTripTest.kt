package com.intrada.android

import com.intrada.android.core.LiveBridge
import com.intrada.android.core.withIds
import com.intrada.ffi.clickTempoWords
import com.intrada.shared.Accidental
import com.intrada.shared.AppEffect
import com.intrada.shared.ClickBarOption
import com.intrada.shared.ClickPreset
import com.intrada.shared.ClickPresetOption
import com.intrada.shared.ClickStart
import com.intrada.shared.CreateItem
import com.intrada.shared.Effect
import com.intrada.shared.Event
import com.intrada.shared.ExerciseLink
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKind
import com.intrada.shared.Key
import com.intrada.shared.Letter
import com.intrada.shared.LibraryItemView
import com.intrada.shared.LinkChange
import com.intrada.shared.Modality
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.PersistenceOutput
import com.intrada.shared.PracticeDefaultsEvent
import com.intrada.shared.Request
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

    // Links cross on a second decoder: a skewed ExerciseLink or link view shows only here (#846,
    // #2248).
    @Test
    fun anExerciseLinkedToTwoPiecesDecodes() {
        val bridge = LiveBridge()
        val nocturne = addItem(bridge, "Nocturne", ItemKind.PIECE)
        val etude = addItem(bridge, "Étude", ItemKind.PIECE)
        val thirds = addItem(bridge, "Thirds", ItemKind.EXERCISE)

        val written: List<ExerciseLink> =
            bridge
                .update(Event.Item(ItemEvent.ChooseExercisePieces(thirds, listOf(nocturne, etude))))
                .mapNotNull {
                    ((it.effect as? Effect.Persistence)?.value as? PersistenceOperation.SaveItems)
                        ?.value
                }
                .single()
                .flatMap { it.exerciseLinks }
        assertEquals(listOf(thirds, thirds), written.map { it.exerciseId })

        val rows =
            libraryChanged(
                bridge.update(
                    Event.Item(ItemEvent.ChangePieceLink(nocturne, LinkChange.Unlink(thirds)))
                )
            )
        assertTrue(rows.single { it.id == nocturne }.linkedExercises.isEmpty())
        assertEquals(
            listOf(thirds),
            rows.single { it.id == etude }.linkedExercises.map { it.id },
        )
        assertEquals(
            listOf("Étude"),
            rows.single { it.id == thirds }.usedIn.filter { it.linked }.map { it.piece?.title },
        )
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
        assertEquals("the tombstone stays hidden", hanon, card.id)
        val usedIn = rows.single { it.id == hanon }.usedIn.single()
        assertTrue(usedIn.linked)
    }

    // A live link and a tombstoned one to the same exercise.
    private fun pieceWithLoadedLinks(hanon: String) =
        Fixtures.item(
            exerciseLinks =
                listOf(
                    ExerciseLink("l1", hanon, 0uL, "2026-10-04T09:00:00Z", null),
                    ExerciseLink("l2", hanon, 1uL, "2026-10-04T09:05:00Z", "2026-10-04T09:05:00Z"),
                )
        )

    // Time away crosses on Kotlin's encoder and comes back on its decoder (#846, #2249).
    @Test
    fun timeAwayRunsThroughThePracticeScreen() {
        val bridge = LiveBridge()
        bridge.update(Event.StartApp)
        val nocturne = addItem(bridge, "Nocturne", ItemKind.PIECE)
        bridge.update(Event.Session(SessionEvent.StartBuilding))
        bridge.update(Event.Session(SessionEvent.AddToSetlist(nocturne)))

        bridge.update(Event.Session(SessionEvent.StartSession("2026-10-04T09:00:00Z")))
        bridge.update(Event.Session(SessionEvent.WentAway("2026-10-04T09:01:00Z")))
        bridge.update(Event.Session(SessionEvent.CameBack("2026-10-04T09:07:00Z")))
        assertEquals(6u, bridge.view().activeSession?.record?.awayOffer?.minutes)
        bridge.update(Event.Session(SessionEvent.LeaveAwayOut))
        bridge.update(
            Event.Session(
                SessionEvent.PrepareReflection(
                    "2026-10-04T09:16:40Z",
                    TempoReading(bpm = 84.toUShort(), clickSounding = false),
                )
            )
        )

        assertEquals(640uL, bridge.view().activeSession?.reflection?.elapsedSecs)
    }

    // The builder rules the core owns cross both ways on Kotlin's codec (#2393).
    @Test
    fun plannedVariationsCrossTheBridge() {
        val bridge = LiveBridge()
        bridge.update(Event.StartApp)
        val nocturne = addItem(bridge, "Nocturne", ItemKind.PIECE)
        bridge.update(
            Event.Item(ItemEvent.UpdateItemVariations(nocturne, emptyList(), listOf("Dotted")))
        )
        val dotted = bridge.view().variations.single { it.label == "Dotted" }.id
        bridge.update(Event.Session(SessionEvent.StartBuilding))
        bridge.update(Event.Session(SessionEvent.AddToSetlist(nocturne)))
        val entryId = bridge.view().buildingSetlist?.entries?.firstOrNull()?.id.orEmpty()
        bridge.update(Event.Session(SessionEvent.SetEntryVariations(entryId, listOf(dotted))))

        val building = bridge.view().buildingSetlist
        assertEquals(listOf(dotted), building?.entries?.firstOrNull()?.plannedVariationIds)
    }

    @Test
    fun theFinishSheetHasOneRowPerPlay() {
        val bridge = LiveBridge()
        bridge.update(Event.StartApp)
        val nocturne = addItem(bridge, "Nocturne", ItemKind.PIECE)
        bridge.update(Event.Session(SessionEvent.StartBuilding))
        bridge.update(Event.Session(SessionEvent.AddToSetlist(nocturne)))

        bridge.update(Event.Session(SessionEvent.StartSession("2026-10-04T09:00:00Z")))
        bridge.update(
            Event.Session(
                SessionEvent.PrepareReflection(
                    "2026-10-04T09:05:00Z",
                    TempoReading(bpm = 84.toUShort(), clickSounding = false),
                )
            )
        )
        val active = checkNotNull(bridge.view().activeSession)
        val rows = checkNotNull(active.record.finish).rows.map { it.playId }
        assertTrue(rows.isNotEmpty())
        assertEquals(active.entries.first().plays.map { it.id }, rows)
    }

    // Real version 2 device bytes: a skewed Vec<u8> would carry nothing and raise nothing (#2791).
    @Test
    fun versionTwoDefaultsCarryIntoTheViewAndAreSavedOnce() {
        val bridge = LiveBridge()
        bridge.update(Event.StartApp)
        val bytes = listOf(0x05, 0x01, 0x00, 0x00, 0x00, 0x01, 0x19, 0x00).map { it.toUByte() }

        val saved =
            bridge
                .update(Event.PracticeDefaults(PracticeDefaultsEvent.Carried(bytes)))
                .mapNotNull {
                    ((it.effect as? Effect.App)?.value as? AppEffect.SavePracticeDefaults)?.value
                }
                .single()

        val shown = bridge.view().practiceDefaults
        assertEquals(ClickStart.TWOANDFOUR, shown.click)
        assertEquals(25.toUShort(), shown.sessionLengthMins)
        assertEquals(shown, saved)
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

    private fun libraryChanged(requests: List<Request>): List<LibraryItemView> =
        requests
            .mapNotNull { ((it.effect as? Effect.App)?.value as? AppEffect.LibraryChanged)?.value }
            .single()
}
