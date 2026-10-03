package com.intrada.android.ui.components

import androidx.compose.ui.graphics.Brush
import com.intrada.android.ui.IntradaGradient
import com.intrada.shared.ItemKind

val ItemKind.bar: Brush
    get() =
        when (this) {
            ItemKind.PIECE -> IntradaGradient.pieceBar
            ItemKind.EXERCISE -> IntradaGradient.exerciseBar
        }

val ItemKind.label: String
    get() =
        when (this) {
            ItemKind.PIECE -> "Piece"
            ItemKind.EXERCISE -> "Exercise"
        }
