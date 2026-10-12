package com.intrada.android

import com.intrada.android.core.LiveBridge
import com.intrada.shared.Effect
import com.intrada.shared.Event
import com.intrada.shared.ExerciseLink
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.PersistenceOutput
import com.intrada.shared.SessionEvent
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

// The core's ruling on "Show suggestion" crosses on Kotlin's decoder (#846, #2521).
class SuggestionRestoreRoundTripTest {
    @Test
    fun showSuggestionFollowsThePlanAndWhatIsUnderWay() {
        val hanon = "01J0000000000000000000HANO"
        val piece =
            Fixtures.item(
                exerciseLinks =
                    listOf(ExerciseLink("l1", hanon, null, 0uL, "2026-10-04T09:00:00Z", null))
            )
        val bridge = LiveBridge()
        val load =
            bridge.update(Event.StartApp).single {
                (it.effect as? Effect.Persistence)?.value == PersistenceOperation.LoadItems
            }
        bridge.resolve(load.id, PersistenceOutput.Items(listOf(piece) + Fixtures.library.drop(1)))

        assertNotNull(bridge.view().upNext)
        assertTrue(bridge.view().showsSuggestionRestore)

        bridge.update(Event.Session(SessionEvent.StartBuilding))
        assertFalse(bridge.view().showsSuggestionRestore)
    }
}
