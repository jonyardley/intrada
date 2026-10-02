package com.intrada.android.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.CustomAccessibilityAction
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.customActions
import androidx.compose.ui.semantics.testTag
import androidx.compose.ui.unit.dp

@Composable
fun GlobalBanner(
    message: String,
    tag: String,
    modifier: Modifier = Modifier,
    onDismiss: (() -> Unit)? = null,
) {
    val text = IntradaFont.bodyMedium.copy(color = IntradaColor.danger)
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
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        BasicText(message, Modifier.weight(1f).padding(vertical = 10.dp), style = text)
        if (onDismiss != null) {
            Box(
                Modifier.size(48.dp).clickable(onClick = onDismiss),
                contentAlignment = Alignment.Center,
            ) {
                BasicText("✕", style = IntradaFont.metaMedium.copy(color = IntradaColor.danger))
            }
        }
    }
}
