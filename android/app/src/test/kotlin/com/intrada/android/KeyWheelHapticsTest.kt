package com.intrada.android

import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.hapticfeedback.HapticFeedback
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Store
import com.intrada.android.ui.Haptic
import com.intrada.android.ui.LibraryAddRoute
import com.intrada.shared.Event
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class KeyWheelHapticsTest {
    @get:Rule val compose = createComposeRule()

    private val fired = mutableListOf<HapticFeedbackType>()
    private val recorder =
        object : HapticFeedback {
            override fun performHapticFeedback(hapticFeedbackType: HapticFeedbackType) {
                fired += hapticFeedbackType
            }
        }

    @Test
    fun aFirstTapTicksAndAFlipOfSpellingTapsHarder() = runTest {
        val store =
            Store(
                LiveBridge(),
                InMemoryItemStore(emptyList()),
                this,
                StandardTestDispatcher(testScheduler),
                log = {},
            )
        store.send(Event.StartApp)
        store.settle()
        compose.setContent {
            CompositionLocalProvider(LocalHapticFeedback provides recorder) {
                LibraryAddRoute(store, onDone = {})
            }
        }

        compose.onNodeWithTag("itemForm.key").performScrollTo().performClick()
        val spoke = compose.onNodeWithTag("itemForm.key.major.6").performScrollTo()
        spoke.performClick()
        spoke.performClick()
        compose.onNodeWithTag("itemForm.key.major.1").performScrollTo().performClick()

        assertEquals(
            listOf(Haptic.SELECTION, Haptic.IMPACT, Haptic.SELECTION).map { it.type },
            fired,
        )
    }
}
