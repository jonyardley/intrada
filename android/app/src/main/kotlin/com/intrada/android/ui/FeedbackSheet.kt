package com.intrada.android.ui

import android.graphics.BitmapFactory
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.Image
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import com.intrada.android.core.FeedbackReport
import com.intrada.android.ui.components.cardSurface

/**
 * Send feedback during the beta (#598): a note, and from a shake the screen the tester was on, sent
 * to Sentry with the build and recent steps.
 */
@Composable
fun FeedbackSheet(
    screenshot: ByteArray?,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
    send: (FeedbackReport) -> Unit = FeedbackReport::send,
) {
    var note by rememberSaveable { mutableStateOf("") }
    var includeScreenshot by rememberSaveable { mutableStateOf(true) }
    val report = FeedbackReport.of(note, screenshot, includeScreenshot)
    // Typed words are unsaved input, so back only closes an empty form.
    BackHandler { if (note.isBlank()) onDismiss() }
    ScreenScaffold(
        "Feedback",
        modifier,
        actions = {
            TextAction("Cancel", "feedback.cancel", onDismiss)
            TextAction(
                "Send",
                "feedback.send",
                {
                    report?.let(send)
                    onDismiss()
                },
                emphasised = true,
                enabled = report != null,
            )
        },
    ) {
        Column(
            Modifier.verticalScroll(rememberScrollState()).padding(IntradaSpacing.card),
            verticalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
        ) {
            Column(Modifier.cardSurface()) {
                FormField(
                    "Note",
                    note,
                    { note = it },
                    "feedback.note",
                    placeholder = "Bug or idea",
                    singleLine = false,
                )
            }
            if (screenshot != null) {
                ScreenshotCard(screenshot, includeScreenshot, { includeScreenshot = it })
            }
        }
    }
}

@Composable
private fun ScreenshotCard(
    screenshot: ByteArray,
    include: Boolean,
    onInclude: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
) {
    val image =
        remember(screenshot) {
            BitmapFactory.decodeByteArray(screenshot, 0, screenshot.size)?.asImageBitmap()
        }
    Column(
        modifier
            .fillMaxWidth()
            .cardSurface()
            .padding(horizontal = IntradaSpacing.card, vertical = IntradaSpacing.cardCompact),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
    ) {
        ToggleRow("Include screenshot", include, "feedback.includeScreenshot", onInclude)
        if (include && image != null) {
            val shape = RoundedCornerShape(IntradaRadius.control)
            Box(Modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
                Image(
                    image,
                    contentDescription = "Screenshot of the screen you were on",
                    modifier =
                        Modifier.heightIn(max = 220.dp)
                            .aspectRatio(image.width.toFloat() / image.height)
                            .clip(shape)
                            .border(1.dp, IntradaColor.divider, shape),
                    contentScale = ContentScale.Fit,
                )
            }
        }
    }
}
