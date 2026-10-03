package com.intrada.android.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp

@Composable
fun ScreenScaffold(
    title: String,
    modifier: Modifier = Modifier,
    content: @Composable BoxScope.() -> Unit,
) {
    Column(
        modifier
            .fillMaxSize()
            .background(IntradaGradient.paper)
            .windowInsetsPadding(WindowInsets.safeDrawing)
    ) {
        BasicText(
            title,
            Modifier.padding(horizontal = IntradaSpacing.card)
                .padding(top = IntradaSpacing.controlGap)
                .semantics { heading() },
            style = IntradaFont.pageTitle.copy(color = IntradaColor.ink),
        )
        Box(
            Modifier.padding(top = IntradaSpacing.cardCompact)
                .fillMaxWidth()
                .height(1.dp)
                .background(IntradaColor.divider)
        )
        Box(Modifier.fillMaxWidth().weight(1f).clipToBounds(), content = content)
    }
}
