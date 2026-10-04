package com.intrada.android.ui

import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.EaseOut
import androidx.compose.animation.core.Easing
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.LinearGradientShader
import androidx.compose.ui.graphics.RadialGradientShader
import androidx.compose.ui.graphics.Shader
import androidx.compose.ui.graphics.ShaderBrush
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.intrada.android.R
import com.intrada.shared.HighlighterColour
import kotlin.math.PI
import kotlin.math.pow
import kotlin.time.Duration
import kotlin.time.Duration.Companion.milliseconds

// Names and values from ios/Intrada/DesignSystem/Theme.swift, held by ThemeTokenParityTest.
object IntradaColor {
    val paperTop = Color(0xFFF7F4EF)

    val cardFill = Color(0xFFFFFFFF)
    val surfaceSunken = Color(0xFFF3EFE8)
    val hairline = Color(0xFFECE6DA)
    val divider = Color(0xFFDDD7CA)

    val ink = Color(0xFF2A2725)
    val inkSecondary = Color(0xFF6E6A66)
    val inkFaintIcon = Color(0xFF8F8070)
    val iconInk = Color(0xFF3B2A1E)

    val accent = ink
    val onAccent = Color(0xFFFFFFFF)
    val marker = Color(0xFFFFE9A3)

    private val markers =
        HighlighterColour.entries.associateWith {
            when (it) {
                HighlighterColour.BUTTER -> marker
                HighlighterColour.CORAL -> Color(0xFFFFB4A2)
                HighlighterColour.MINT -> Color(0xFFB9EBD3)
                HighlighterColour.SKY -> Color(0xFFA9D8FF)
                HighlighterColour.LAVENDER -> Color(0xFFD6CCF5)
                HighlighterColour.SAGE -> Color(0xFFCFDDB0)
                HighlighterColour.PEACH -> Color(0xFFFFD7B0)
                HighlighterColour.POWDER -> Color(0xFFC7DCE8)
            }
        }

    fun marker(colour: HighlighterColour): Color = markers.getValue(colour)

    val onMarker = ink
    val danger = Color(0xFF9C4A3A)
    val dangerWash = danger.copy(alpha = 0.10f)
    val dangerBanner = danger.copy(alpha = 0.12f)
    val dangerEdge = danger.copy(alpha = 0.25f)
    val shadow = ink.copy(alpha = 0.05f)
    val buttonShadow = ink.copy(alpha = 0.06f)
    val sheetScrim = Color.Black.copy(alpha = 0.2f)

    val tabBarFill = Color(0xFFF3EFE8)

    val pieceBadgeBg = Color(0xFFCBD6E0)
    val pieceBadgeFg = ink
    val exerciseBadgeBg = Color(0xFFDCD3C0)
    val exerciseBadgeFg = ink

    val success = Color(0xFF4C6B3F)
    val masteryFill = ink
    val masteryTrack = Color(0xFFEDE8DC)
    val dialTrack = Color(0xFFEDE8DC)
    val timerTrack = Color(0xFFEDE8DC)
    val consistencyTrack = Color(0xFFEDE8DC)
    val repMissedFg = Color(0xFF756A5C)
    val repMissedBg = Color(0xFFF3EFE8)
    val repCleanFg = success
    val repCleanBg = Color(0xFFEDF1EA)
    val repCleanBorder = Color(0xFFD3DDCC)
    val slotOutline = Color(0xFFDDD7CA)
    val addDashOutline = Color(0xFFC9BFB0)
    val inkFainter = Color(0xFFC2B8AA)
    val playerBgTop = Color(0xFFFBFAF7)
    val playerBgMid = Color(0xFFF7F4EF)
    val playerBgBottom = Color(0xFFEFEAE1)
    val heroGradientTop = ink

    private val heroGradientBottoms =
        HighlighterColour.entries.associateWith {
            when (it) {
                HighlighterColour.BUTTER -> Color(0xFF4F3B28)
                HighlighterColour.CORAL -> Color(0xFF4C2C24)
                HighlighterColour.MINT -> Color(0xFF244C39)
                HighlighterColour.SKY -> Color(0xFF243A4C)
                HighlighterColour.LAVENDER -> Color(0xFF2E244C)
                HighlighterColour.SAGE -> Color(0xFF3D4923)
                HighlighterColour.PEACH -> Color(0xFF4C3824)
                HighlighterColour.POWDER -> Color(0xFF243D4C)
            }
        }

    fun heroGradientBottom(colour: HighlighterColour): Color = heroGradientBottoms.getValue(colour)

    val onHeroExercise = exerciseBadgeBg
    val onHeroPiece = pieceBadgeBg
    val celebrationBg = ink
    val celebrationInk = paperTop
    val viewerBackdrop = Color(0xFF1A1917)
}

// iOS splits these across LinearGradient and RadialGradient; a one-colour gradient is a solid fill.
object IntradaGradient {
    val paper: Brush = SolidColor(IntradaColor.paperTop)
    val inkBar: Brush = SolidColor(IntradaColor.ink)
    val celebration: Brush = SolidColor(IntradaColor.celebrationBg)
    val pieceBar: Brush = SolidColor(IntradaColor.pieceBadgeBg)
    val exerciseBar: Brush = SolidColor(IntradaColor.exerciseBadgeBg)
    val ringSweep: Brush = SolidColor(IntradaColor.ink)

    fun practiceHero(colour: HighlighterColour): Brush =
        TopTrailingToBottomLeading(
            listOf(IntradaColor.heroGradientTop, IntradaColor.heroGradientBottom(colour))
        )

    private val playerPaperCentre = Offset(0.5f, 0.22f)

    fun playerPaper(density: Density): Brush =
        PlayerPaper(
            listOf(IntradaColor.playerBgTop, IntradaColor.playerBgMid, IntradaColor.playerBgBottom),
            centre = playerPaperCentre,
            radius = with(density) { 440.dp.toPx() },
        )
}

private data class TopTrailingToBottomLeading(val colors: List<Color>) : ShaderBrush() {
    override fun createShader(size: Size): Shader =
        LinearGradientShader(Offset(size.width, 0f), Offset(0f, size.height), colors)
}

private data class PlayerPaper(val colors: List<Color>, val centre: Offset, val radius: Float) :
    ShaderBrush() {
    override fun createShader(size: Size): Shader =
        RadialGradientShader(Offset(size.width * centre.x, size.height * centre.y), radius, colors)
}

object IntradaFont {
    object Hanken {
        val regular = Font(R.font.hanken_grotesk_regular, FontWeight.Normal)
        val medium = Font(R.font.hanken_grotesk_medium, FontWeight.Medium)
        val semibold = Font(R.font.hanken_grotesk_semibold, FontWeight.SemiBold)
        val bold = Font(R.font.hanken_grotesk_bold, FontWeight.Bold)
    }

    object Mono {
        val regular = Font(R.font.dm_mono_regular, FontWeight.Normal)
    }

    private val hanken = FontFamily(Hanken.regular, Hanken.medium, Hanken.semibold, Hanken.bold)
    private val mono = FontFamily(Mono.regular)

    private fun hanken(weight: FontWeight, size: TextUnit) =
        TextStyle(fontFamily = hanken, fontWeight = weight, fontSize = size)

    const val pageTitleSize = 30f
    const val cardTitleSize = 17f

    val pageTitle = hanken(FontWeight.SemiBold, pageTitleSize.sp)
    val title = hanken(FontWeight.SemiBold, 20.sp)
    val cardTitle = hanken(FontWeight.SemiBold, cardTitleSize.sp)
    val body = hanken(FontWeight.Normal, 17.sp)
    val bodyMedium = hanken(FontWeight.Medium, 17.sp)
    val label = hanken(FontWeight.Medium, 15.sp)
    val secondary = hanken(FontWeight.Normal, 15.sp).copy(fontFeatureSettings = "tnum")
    val figure = TextStyle(fontFamily = mono, fontWeight = FontWeight.Normal, fontSize = 15.sp)
    val button = hanken(FontWeight.Bold, 15.sp)
    val segment = hanken(FontWeight.Medium, 15.sp)
    val small = hanken(FontWeight.Normal, 13.sp)
    val smallMedium = hanken(FontWeight.Medium, 13.sp)
    val badge = hanken(FontWeight.SemiBold, 13.sp)

    fun timer(size: TextUnit = 56.sp) = hanken(FontWeight.SemiBold, size)

    fun scoreNumeral(size: TextUnit) = hanken(FontWeight.SemiBold, size)

    val chart = TextStyle(fontFamily = FontFamily.Monospace, fontSize = 13.sp)
    val chartEditor = TextStyle(fontFamily = FontFamily.Monospace, fontSize = 17.sp)
}

object IntradaSpacing {
    val controlGap = 8.dp
    val cardCompact = 12.dp
    val card = 16.dp
    val section = 24.dp
}

object IntradaRadius {
    val card = 3.dp
    val control = 3.dp
    val badge = 3.dp
    val panel = 3.dp
    val hero = 3.dp
    val pill = 999.dp
}

object IntradaGlyph {
    val bar = 36.dp
    val tile = 56.dp
    val hero = 88.dp
}

class IntradaIconSize private constructor(val points: Dp, val maxPoints: Dp) {
    companion object {
        val badge = IntradaIconSize(9.dp, 13.dp)
        val caption = IntradaIconSize(11.dp, 16.dp)
        val inline = IntradaIconSize(15.dp, 18.dp)
        val control = IntradaIconSize(20.dp, 26.dp)
        val large = IntradaIconSize(28.dp, 36.dp)
        val transport = IntradaIconSize(32.dp, 40.dp)
        val hero = IntradaIconSize(38.dp, 56.dp)
    }
}

object IntradaOpacity {
    const val wash = 0.12f
    const val dimmed = 0.5f
    const val soft = 0.6f
    const val secondary = 0.75f
    const val strong = 0.8f
}

data class IntradaShadow(val color: Color, val radius: Dp, val y: Dp) {
    companion object {
        val none = IntradaShadow(Color.Transparent, 0.dp, 0.dp)
        val card = IntradaShadow(IntradaColor.shadow, 1.dp, 1.dp)
        val button = IntradaShadow(IntradaColor.buttonShadow, 1.dp, 1.dp)
        val lifted = IntradaShadow(IntradaColor.shadow, 6.dp, 3.dp)
        val glow = IntradaShadow(IntradaColor.accent.copy(alpha = 0.4f), 6.dp, 4.dp)
        val transport = IntradaShadow(IntradaColor.ink.copy(alpha = 0.18f), 14.dp, 6.dp)
        val heroButton = IntradaShadow(Color.Black.copy(alpha = 0.25f), 16.dp, 8.dp)
        val hero = IntradaShadow(Color.Black.copy(alpha = 0.18f), 20.dp, 10.dp)
        val appIcon = IntradaShadow(IntradaColor.ink.copy(alpha = IntradaOpacity.wash), 10.dp, 8.dp)
    }
}

// SwiftUI's spring(response:dampingFraction:); Compose's spring() takes the same damping and this
// stiffness.
data class IntradaSpring(val response: Float, val dampingFraction: Float) {
    val stiffness: Float
        get() = (2 * PI / response).pow(2).toFloat()
}

data class IntradaTiming(
    val duration: Duration,
    val easing: Easing,
    val delay: Duration = Duration.ZERO,
)

object IntradaMotion {
    val standard = IntradaSpring(response = 0.35f, dampingFraction = 0.85f)
    val snappy = IntradaSpring(response = 0.28f, dampingFraction = 0.9f)
    val gentle = IntradaSpring(response = 0.45f, dampingFraction = 0.82f)

    val fadeUpDuration = 500.milliseconds
    val fadeUpStagger = 60.milliseconds
    val fadeUpOffset = 12.dp

    val barGrow = IntradaTiming(600.milliseconds, CubicBezierEasing(0.2f, 0.8f, 0.3f, 1f))
    val barGrowStagger = 60.milliseconds
    val countUpDuration = 1500.milliseconds
    val pop = IntradaSpring(response = 0.35f, dampingFraction = 0.62f)
    val reduceFade = 150.milliseconds

    fun fadeUp(index: Int) = IntradaTiming(fadeUpDuration, EaseOut, fadeUpStagger * index)
}
