package com.intrada.android

import android.graphics.Bitmap
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.core.graphics.createBitmap
import com.intrada.android.core.FeedbackReport
import com.intrada.android.ui.FeedbackSheet
import java.io.ByteArrayOutputStream
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class FeedbackSheetTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun sendWaitsForANoteThenSendsItWithoutTheScreenshotWhenSwitchedOff() {
        val sent = mutableListOf<FeedbackReport>()
        var dismissed = 0
        compose.setContent {
            FeedbackSheet(feedbackScreenshot(), onDismiss = { dismissed++ }, send = { sent += it })
        }

        compose.onNodeWithTag("feedback.send").assertIsNotEnabled()
        compose.onNodeWithTag("feedback.note").performTextInput("  Timer froze  ")
        compose.onNodeWithTag("feedback.includeScreenshot").performClick()
        compose.onNodeWithTag("feedback.send").assertIsEnabled().performClick()

        assertEquals("Timer froze", sent.single().message)
        assertNull(sent.single().screenshot)
        assertEquals(1, dismissed)
    }
}

fun feedbackScreenshot(): ByteArray {
    val bitmap = createBitmap(108, 240).apply { eraseColor(0xFFB8C4D0.toInt()) }
    return ByteArrayOutputStream()
        .also { bitmap.compress(Bitmap.CompressFormat.PNG, 100, it) }
        .toByteArray()
}
