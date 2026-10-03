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
import androidx.compose.ui.unit.dp
import com.intrada.android.R
import com.intrada.android.ui.components.scaled

private val touchTarget = 48.dp

@Composable
fun GlobalBanner(
    message: String,
    tag: String,
    modifier: Modifier = Modifier,
    onDismiss: (() -> Unit)? = null,
) {
    val ink = IntradaColor.danger
    val text = IntradaFont.bodyMedium.copy(color = ink)
    val firstLine =
        with(LocalDensity.current) { rememberTextMeasurer().measure("A", text).size.height.toDp() }
    Row(
        modifier
            .fillMaxWidth()
            .background(IntradaColor.dangerBanner)
            .clearAndSetSemantics {
                testTag = tag
                contentDescription = message
                if (onDismiss != null) {
                    customActions =
                        listOf(
                            CustomAccessibilityAction("Dismiss") {
                                onDismiss()
                                true
                            }
                        )
                }
            }
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
                    BannerIcon(R.drawable.ic_warning_triangle, IntradaIconSize.control, ink)
                }
                BasicText(message, Modifier.weight(1f), style = text)
            }
        }
        if (onDismiss != null) {
            Box(
                Modifier.size(touchTarget).clickable(onClick = onDismiss),
                contentAlignment = Alignment.Center,
            ) {
                BannerIcon(R.drawable.ic_close, IntradaIconSize.caption, ink)
            }
        }
    }
}

@Composable
private fun BannerIcon(
    @DrawableRes id: Int,
    size: IntradaIconSize,
    ink: Color,
) {
    Image(
        painterResource(id),
        contentDescription = null,
        modifier = Modifier.size(size.scaled()),
        colorFilter = ColorFilter.tint(ink),
    )
}
