package com.intrada.android.ui.components

import androidx.compose.runtime.Composable
import androidx.compose.runtime.ReadOnlyComposable
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.min
import com.intrada.android.ui.IntradaIconSize

@Composable
@ReadOnlyComposable
internal fun IntradaIconSize.scaled(): Dp = min(points * LocalDensity.current.fontScale, maxPoints)
