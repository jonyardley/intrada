package com.intrada.android

import com.intrada.android.core.LiveBridge
import com.intrada.shared.Accidental
import com.intrada.shared.ActiveSession
import com.intrada.shared.AppEffect
import com.intrada.shared.Effect
import com.intrada.shared.EntryStatus
import com.intrada.shared.Event
import com.intrada.shared.FocusKind
import com.intrada.shared.IntentionFocus
import com.intrada.shared.ItemKind
import com.intrada.shared.Key
import com.intrada.shared.Letter
import com.intrada.shared.Modality
import com.intrada.shared.PlannedItem
import com.intrada.shared.Play
import com.intrada.shared.Segment
import com.intrada.shared.SessionEvent
import com.intrada.shared.SetlistEntry
import org.junit.Assert.assertEquals
import org.junit.Test

class PracticeInProgressRoundTripTest {
    // A practice in progress with every plan field set crosses Kotlin's encoder into the core
    // and back out as the saved copy (#846, #2582).
    @Test
    fun aPracticeInProgressKeepsEveryPlanField() {
        val plan =
            PlannedItem(
                itemId = "i1",
                itemType = ItemKind.EXERCISE,
                intention = "A at 84",
                focus = IntentionFocus(FocusKind.CLEANREPS, "s-b", 5.toUShort()),
                durationSecs = 1200u,
                groupId = "block-1",
                variationIds = listOf("v-a", "v-b"),
                repTarget = 6.toUByte(),
                segments = listOf(Segment("s-a", 600u), Segment("s-b", 600u)),
                key = Key(Letter.G, Accidental.NATURAL, Modality.MAJOR),
            )
        val at = "2026-10-04T09:00:00Z"
        val entry =
            SetlistEntry(
                id = "e1",
                plan = plan,
                itemTitle = "Nocturne",
                position = 0uL,
                durationSecs = 0uL,
                status = EntryStatus.NOTATTEMPTED,
                notes = null,
                plays =
                    listOf(
                        Play(
                            id = "p1",
                            variationIds = emptyList(),
                            startedAt = at,
                            seconds = 0uL,
                            tempoChanges = emptyList(),
                            away = emptyList(),
                        )
                    ),
                intentionMet = null,
                felt = null,
                gotInTheWay = emptyList(),
                notePoints = emptyList(),
            )
        val bridge = LiveBridge()
        bridge.update(Event.StartApp)
        bridge.update(
            Event.Session(
                SessionEvent.RecoverSession(
                    ActiveSession("v9", listOf(entry), 0uL, at, at, null, null),
                    at,
                )
            )
        )

        val saved =
            bridge
                .update(Event.Session(SessionEvent.WentAway("2026-10-04T09:01:00Z")))
                .firstNotNullOf {
                    ((it.effect as? Effect.App)?.value as? AppEffect.SaveSessionInProgress)?.value
                }
        assertEquals(plan, saved.entries.single().plan)
        assertEquals(EntryStatus.NOTATTEMPTED, saved.entries.single().status)
    }
}
