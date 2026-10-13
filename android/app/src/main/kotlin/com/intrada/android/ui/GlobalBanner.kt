package com.intrada.android.ui

import androidx.annotation.DrawableRes
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.CustomAccessibilityAction
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.customActions
import androidx.compose.ui.semantics.testTag
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.intrada.android.R
import com.intrada.android.ui.components.scaled

private val touchTarget = 48.dp

/** `DANGER` is a refusal or a failed write; `NOTICE` is a true outcome that is not a failure. */
enum class BannerTone {
    DANGER,
    NOTICE,
}

/** A one-word way back, such as Undo, shown before the dismiss button. */
class BannerAction(val label: String, val perform: () -> Unit)

@Composable
fun GlobalBanner(
    message: String,
    tag: String,
    modifier: Modifier = Modifier,
    tone: BannerTone = BannerTone.DANGER,
    action: BannerAction? = null,
    onDismiss: (() -> Unit)? = null,
) {
    val notice = tone == BannerTone.NOTICE
    val ink = if (notice) IntradaColor.inkSecondary else IntradaColor.danger
    val text = IntradaFont.bodyMedium.copy(color = ink)
    val density = LocalDensity.current
    val measurer = rememberTextMeasurer()
    val firstLine =
        remember(text, density) { with(density) { measurer.measure("A", text).size.height.toDp() } }
    val glyph = with(density) { text.fontSize.toDp() }
    Row(
        modifier
            .fillMaxWidth()
            .background(if (notice) IntradaColor.cardFill else IntradaColor.dangerBanner)
            .bannerSemantics(tag, if (notice) "Note, $message" else message, action, onDismiss)
            .padding(
                start = IntradaSpacing.card,
                end = if (onDismiss == null) IntradaSpacing.card else 0.dp,
            ),
        verticalAlignment = Alignment.Top,
    ) {
        Box(
            Modifier.weight(1f).heightIn(min = touchTarget),
            contentAlignment = Alignment.CenterStart,
        ) {
            Row(
                Modifier.padding(vertical = 10.dp),
                horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
                verticalAlignment = Alignment.Top,
            ) {
                Box(Modifier.height(firstLine), contentAlignment = Alignment.Center) {
                    BannerIcon(
                        if (notice) R.drawable.ic_info else R.drawable.ic_warning_triangle,
                        glyph,
                        ink,
                    )
                }
                BasicText(message, Modifier.weight(1f), style = text)
            }
        }
        action?.let { BannerActionButton(it) }
        if (onDismiss != null) {
            Box(
                Modifier.size(touchTarget).clickable(onClick = onDismiss),
                contentAlignment = Alignment.Center,
            ) {
                BannerIcon(R.drawable.ic_close, IntradaIconSize.caption.scaled(), ink)
            }
        }
    }
}

private fun Modifier.bannerSemantics(
    tag: String,
    spoken: String,
    action: BannerAction?,
    onDismiss: (() -> Unit)?,
): Modifier = clearAndSetSemantics {
    testTag = tag
    contentDescription = spoken
    customActions =
        listOfNotNull(
            action?.let {
                CustomAccessibilityAction(it.label) {
                    it.perform()
                    true
                }
            },
            onDismiss?.let {
                CustomAccessibilityAction("Dismiss") {
                    it()
                    true
                }
            },
        )
}

@Composable
private fun BannerActionButton(action: BannerAction) {
    Box(
        Modifier.heightIn(min = touchTarget)
            .clickable(onClick = action.perform)
            .padding(horizontal = IntradaSpacing.controlGap),
        contentAlignment = Alignment.Center,
    ) {
        BasicText(action.label, style = IntradaFont.button.copy(color = IntradaColor.ink))
    }
}

@Composable
private fun BannerIcon(
    @DrawableRes id: Int,
    size: Dp,
    ink: Color,
) {
    Image(
        painterResource(id),
        contentDescription = null,
        modifier = Modifier.size(size),
        colorFilter = ColorFilter.tint(ink),
    )
}
