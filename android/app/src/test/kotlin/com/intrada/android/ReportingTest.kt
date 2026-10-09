package com.intrada.android

import com.intrada.android.core.FeedbackReport
import com.intrada.android.core.ShakeDetector
import com.intrada.android.core.releaseName
import com.intrada.android.core.stepName
import com.intrada.shared.CreateItem
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKind
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class ReportingTest {
    @Test
    fun theReleaseIsNamedAsOnIos() {
        assertEquals("com.intrada.android@0.1.0+1", releaseName("com.intrada.android", "0.1.0", 1))
    }

    @Test
    fun aMissingPartLeavesTheReleaseToTheSdk() {
        assertNull(releaseName(null, "0.1.0", 1))
        assertNull(releaseName("com.intrada.android", "", 1))
        assertNull(releaseName("com.intrada.android", "0.1.0", null))
    }

    @Test
    fun aStepNamesTheEventButNotWhatItCarries() {
        val add =
            Event.Item(
                ItemEvent.Add(
                    CreateItem(
                        title = "Clair de Lune",
                        kind = ItemKind.PIECE,
                        tags = emptyList(),
                        variationLabels = emptyList(),
                    )
                )
            )

        assertEquals("Item.Add", stepName(add))
        assertEquals("ClearError", stepName(Event.ClearError))
    }

    @Test
    fun theNoteIsTrimmedAndABlankOneSendsNothing() {
        assertEquals("Timer froze", FeedbackReport.of("  Timer froze \n", null, true)?.message)
        assertNull(FeedbackReport.of(" \n ", SCREENSHOT, true))
    }

    @Test
    fun theScreenshotGoesOnlyWhenIncluded() {
        assertArrayEquals(SCREENSHOT, FeedbackReport.of("Bug", SCREENSHOT, true)?.screenshot)
        assertNull(FeedbackReport.of("Bug", SCREENSHOT, false)?.screenshot)
    }

    @Test
    fun aJoltPastTheThresholdIsAShakeOnceASecond() {
        val shake = ShakeDetector()

        assertFalse(shake.feed(0f, 0f, 9.8f, atMillis = 0))
        assertTrue(shake.feed(30f, 0f, 9.8f, atMillis = 100))
        assertFalse(shake.feed(30f, 0f, 9.8f, atMillis = 600))
        assertTrue(shake.feed(30f, 0f, 9.8f, atMillis = 1_100))
    }

    private companion object {
        val SCREENSHOT = byteArrayOf(1, 2, 3)
    }
}
