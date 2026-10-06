package com.intrada.android.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp

@Composable
fun ScreenScaffold(
    title: String,
    modifier: Modifier = Modifier,
    actions: @Composable RowScope.() -> Unit = {},
    content: @Composable BoxScope.() -> Unit,
) {
    Column(
        modifier
            .fillMaxSize()
            .background(IntradaGradient.paper)
            .windowInsetsPadding(WindowInsets.safeDrawing)
    ) {
        Row(
            Modifier.fillMaxWidth()
                .padding(horizontal = IntradaSpacing.card)
                .padding(top = IntradaSpacing.controlGap),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            BasicText(
                title,
                Modifier.weight(1f).semantics { heading() },
                style = IntradaFont.pageTitle.copy(color = IntradaColor.ink),
            )
            actions()
        }
        Box(
            Modifier.padding(top = IntradaSpacing.cardCompact)
                .fillMaxWidth()
                .height(1.dp)
                .background(IntradaColor.divider)
        )
        Box(Modifier.fillMaxWidth().weight(1f).clipToBounds(), content = content)
    }
}
