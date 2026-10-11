package com.intrada.android

import androidx.activity.ComponentActivity
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.v2.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.ui.AppFrame
import com.intrada.android.ui.AppTab
import com.intrada.shared.Event
import com.intrada.shared.FirstRunEvent
import kotlinx.coroutines.test.runTest
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class StartHereFrameTest {
    @get:Rule val compose = createAndroidComposeRule<ComponentActivity>()

    @Test
    fun theFirstStepOnThePracticeTabOpensTheAddForm() = runTest {
        val store = openedStore(InMemoryItemStore())
        store.send(Event.FirstRun(FirstRunEvent.SkipWelcome))
        store.settle()
        compose.setContent { AppFrame(store) }

        compose.onNodeWithTag(AppTab.PRACTICE.tag).performClick()
        compose.onNodeWithTag("practice.startHere.next").performScrollTo().performClick()

        compose.onNodeWithTag("itemForm.confirm").assertIsDisplayed()
    }
}
