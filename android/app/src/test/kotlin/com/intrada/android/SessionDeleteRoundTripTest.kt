package com.intrada.android

import com.intrada.android.core.LiveBridge
import com.intrada.shared.Effect
import com.intrada.shared.Event
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.SessionEvent
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

/** A saved session's delete, its undo and the undo banner, through the real bridge (#2581). */
class SessionDeleteRoundTripTest {
    // A deleted session's operations decode here as well as in Swift (#2581).
    @Test
    fun aSessionDeleteAndItsUndoCrossTheBridge() {
        val bridge = LiveBridge()
        bridge.update(Event.LoadSampleData)

        val deleted =
            bridge
                .update(
                    Event.Session(
                        SessionEvent.DeleteSaved("sample-session-3d", "2026-10-13T09:00:00Z")
                    )
                )
                .mapNotNull { (it.effect as? Effect.Persistence)?.value }
        assertEquals(
            listOf(PersistenceOperation.DeleteSession("sample-session-3d", "2026-10-13T09:00:00Z")),
            deleted,
        )
        assertEquals("Session deleted", bridge.view().undoDelete)

        val restored =
            bridge
                .update(Event.Session(SessionEvent.UndoDelete("2026-10-13T09:00:05Z")))
                .mapNotNull { (it.effect as? Effect.Persistence)?.value }
        assertEquals(
            listOf(
                PersistenceOperation.RestoreSession("sample-session-3d", "2026-10-13T09:00:05Z")
            ),
            restored,
        )
        assertNull(bridge.view().undoDelete)
    }

    // A closed undo banner leaves the delete standing (#2581).
    @Test
    fun dismissingTheUndoEndsTheOfferAcrossTheBridge() {
        val bridge = LiveBridge()
        bridge.update(Event.LoadSampleData)
        bridge.update(
            Event.Session(SessionEvent.DeleteSaved("sample-session-3d", "2026-10-13T09:00:00Z"))
        )
        assertEquals("Session deleted", bridge.view().undoDelete)

        bridge.update(Event.Session(SessionEvent.DismissUndoDelete))
        assertNull(bridge.view().undoDelete)

        val undone =
            bridge.update(Event.Session(SessionEvent.UndoDelete("2026-10-13T09:00:05Z"))).filter {
                it.effect is Effect.Persistence
            }
        assertEquals(emptyList<Any>(), undone)
    }
}
