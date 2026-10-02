package com.intrada.android.ui

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

// Token names and values from ios/Intrada/DesignSystem/Theme.swift; only what the Library needs.
object IntradaColor {
    val paperTop = Color(0xFFF7F4EF)
    val cardFill = Color(0xFFFFFFFF)
    val hairline = Color(0xFFECE6DA)
    val ink = Color(0xFF2A2725)
    val inkSecondary = Color(0xFF6E6A66)
    val pieceBar = Color(0xFFCBD6E0)
    val exerciseBar = Color(0xFFDCD3C0)
}

// System faces in place of Hanken Grotesk and DM Mono, which the Android app does not bundle yet.
object IntradaFont {
    val pageTitle =
        TextStyle(
            fontFamily = FontFamily.SansSerif,
            fontWeight = FontWeight.SemiBold,
            fontSize = 32.sp,
        )

    val cardTitle =
        TextStyle(
            fontFamily = FontFamily.SansSerif,
            fontWeight = FontWeight.SemiBold,
            fontSize = 18.sp,
        )

    val subtitle = TextStyle(fontFamily = FontFamily.Monospace, fontSize = 14.sp)
    val body = TextStyle(fontFamily = FontFamily.SansSerif, fontSize = 16.sp)
}

object IntradaSpacing {
    val cardCompact = 12.dp
    val card = 16.dp
}

object IntradaRadius {
    val card = 3.dp
}
