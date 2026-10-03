package com.intrada.android

import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.test.assertContentDescriptionEquals
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import com.intrada.android.ui.GlobalBanner
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class GlobalBannerTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun talkBackReadsTheMessageAndOffersDismiss() {
        var dismissed = 0
        compose.setContent { GlobalBanner(MESSAGE, tag = TAG, onDismiss = { dismissed++ }) }

        val banner = compose.onNodeWithTag(TAG)
        banner.assertContentDescriptionEquals(MESSAGE)
        val dismiss =
            banner.fetchSemanticsNode().config[SemanticsActions.CustomActions].single {
                it.label == "Dismiss"
            }
        compose.runOnIdle { dismiss.action() }

        assertEquals(1, dismissed)
    }

    private companion object {
        const val MESSAGE = "Couldn't delete that item."
        const val TAG = "banner.error"
    }
}
