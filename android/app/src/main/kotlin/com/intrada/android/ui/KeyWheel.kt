package com.intrada.android.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.intrada.android.R
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.cardShadow
import com.intrada.android.ui.components.scaled
import com.intrada.ffi.WheelMode
import com.intrada.ffi.WheelSelection
import com.intrada.shared.Key
import kotlin.math.atan2
import kotlin.math.cos
import kotlin.math.roundToInt
import kotlin.math.sin

@Composable
internal fun KeyPicker(
    key: Key?,
    onKey: (Key?) -> Unit,
    modifier: Modifier = Modifier,
    initiallyExpanded: Boolean = false,
) {
    var expanded by rememberSaveable { mutableStateOf(initiallyExpanded) }
    val selection = key?.let(::wheelSelection)
    Column(modifier.fillMaxWidth()) {
        KeyRow(key, selection, expanded, { expanded = !expanded }, { onKey(null) })
        if (expanded) {
            Box(
                Modifier.fillMaxWidth().padding(top = IntradaSpacing.cardCompact, bottom = 20.dp),
                contentAlignment = Alignment.Center,
            ) {
                KeyWheel(selection) { ring, mode -> keyAfterTap(key, ring, mode)?.let(onKey) }
            }
        }
    }
}

@Composable
private fun KeyRow(
    key: Key?,
    selection: WheelSelection?,
    expanded: Boolean,
    onToggle: () -> Unit,
    onClear: () -> Unit,
) {
    val display = key?.let(::keyDisplay)
    Row(
        Modifier.fillMaxWidth()
            .clickable(role = Role.Button, onClick = onToggle)
            .semantics(mergeDescendants = true) {
                contentDescription =
                    if (selection != null)
                        "Key, ${spokenTonic(selection.spelling)} ${selection.mode.word}"
                    else "Key, ${display ?: "no key selected"}"
                stateDescription = if (expanded) "Wheel open" else "Wheel closed"
            }
            .testTag("itemForm.key")
            .padding(start = IntradaSpacing.card, end = IntradaSpacing.controlGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(
            Modifier.weight(1f).padding(vertical = IntradaSpacing.cardCompact),
            verticalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            FieldLabel("Key")
            BasicText(
                display ?: "Select a key",
                style =
                    IntradaFont.body.copy(
                        color =
                            if (display != null) IntradaColor.accent else IntradaColor.inkSecondary
                    ),
            )
        }
        if (key != null) {
            IconAction(
                R.drawable.ic_close,
                "Clear key",
                onClear,
                Modifier.testTag("itemForm.key.clear"),
                tint = IntradaColor.inkFaintIcon,
            )
        }
        Image(
            painterResource(if (expanded) R.drawable.ic_chevron_up else R.drawable.ic_chevron_down),
            contentDescription = null,
            modifier =
                Modifier.padding(start = IntradaSpacing.cardCompact)
                    .size(IntradaIconSize.inline.scaled()),
            colorFilter = ColorFilter.tint(IntradaColor.inkFaintIcon),
        )
    }
}

private val WHEEL = 300.dp
private val HUB = 120.dp
private val MINOR_INNER = 60.dp
private val RING_SPLIT = 105.dp
private val MAJOR_OUTER = 150.dp
private val LABEL_BOX = 48.dp
private const val SPOKES = 12
private const val SPOKE_DEGREES = 30f
private const val TOP_DEGREES = 270f
private const val FULL_TURN = 360f

private fun spokeDegrees(ring: Int) = TOP_DEGREES + SPOKE_DEGREES * ring

private fun pointOn(radius: Dp, ring: Int): Pair<Dp, Dp> {
    val radians = Math.toRadians(spokeDegrees(ring).toDouble())
    return (WHEEL / 2 + radius * cos(radians).toFloat()) to
        (WHEEL / 2 + radius * sin(radians).toFloat())
}

/**
 * The two-ring circle of fifths, major outside and minor inside, C at the top. [selection] is what
 * the core says the form's key lights; a tap reports its spoke and ring.
 */
@Composable
private fun KeyWheel(selection: WheelSelection?, onTap: (Int, WheelMode) -> Unit) {
    fun chosen(ring: Int, mode: WheelMode) =
        selection?.takeIf { it.ring.toInt() == ring && it.mode == mode }?.spelling
    Box(
        Modifier.size(WHEEL).pointerInput(Unit) {
            detectTapGestures { tap ->
                spokeAt(tap - Offset(size.width / 2f, size.height / 2f))?.let { (ring, mode) ->
                    onTap(ring, mode)
                }
            }
        }
    ) {
        Canvas(Modifier.size(WHEEL)) { drawWedges { ring, mode -> chosen(ring, mode) != null } }
        Box(
            Modifier.offset(WHEEL / 2 - HUB / 2, WHEEL / 2 - HUB / 2)
                .size(HUB)
                .cardShadow(CircleShape)
                .background(IntradaColor.cardFill, CircleShape)
                .border(1.dp, IntradaColor.hairline, CircleShape),
            contentAlignment = Alignment.Center,
        ) {
            Hub(selection)
        }
        for (ring in 0 until SPOKES) {
            for (mode in WheelMode.entries) {
                WedgeLabel(ring, mode, chosen(ring, mode)) { onTap(ring, mode) }
            }
        }
    }
}

private fun Density.spokeAt(fromCentre: Offset): Pair<Int, WheelMode>? {
    val radius = fromCentre.getDistance()
    val mode =
        when {
            radius < MINOR_INNER.toPx() || radius > MAJOR_OUTER.toPx() -> return null
            radius < RING_SPLIT.toPx() -> WheelMode.MINOR
            else -> WheelMode.MAJOR
        }
    val degrees = Math.toDegrees(atan2(fromCentre.y, fromCentre.x).toDouble()).toFloat()
    val clockwise = ((degrees - TOP_DEGREES) % FULL_TURN + FULL_TURN) % FULL_TURN
    return (clockwise / SPOKE_DEGREES).roundToInt() % SPOKES to mode
}

private fun DrawScope.drawWedges(selected: (Int, WheelMode) -> Boolean) {
    val centre = Offset(size.width / 2, size.height / 2)
    for (ring in 0 until SPOKES) {
        for (mode in WheelMode.entries) {
            val major = mode == WheelMode.MAJOR
            val outer = (if (major) MAJOR_OUTER else RING_SPLIT).toPx()
            val inner = (if (major) RING_SPLIT else MINOR_INNER).toPx()
            val start = spokeDegrees(ring) - SPOKE_DEGREES / 2
            val wedge =
                Path().apply {
                    arcTo(Rect(centre, outer), start, SPOKE_DEGREES, true)
                    arcTo(Rect(centre, inner), start + SPOKE_DEGREES, -SPOKE_DEGREES, false)
                    close()
                }
            val fill =
                when {
                    selected(ring, mode) -> IntradaColor.accent
                    major -> IntradaColor.cardFill
                    else -> IntradaColor.surfaceSunken
                }
            drawPath(wedge, fill)
            drawPath(wedge, IntradaColor.hairline, style = Stroke(1.dp.toPx()))
        }
    }
}

@Composable
private fun Hub(selection: WheelSelection?) {
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        if (selection != null) {
            BasicText(
                prettify(selection.spelling),
                style = IntradaFont.title.copy(color = IntradaColor.ink),
            )
            BasicText(
                selection.mode.word,
                style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
            )
        } else {
            BasicText("♪", style = IntradaFont.title.copy(color = IntradaColor.inkFaintIcon))
            BasicText(
                "Select a key",
                style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
            )
        }
    }
}

@Composable
private fun WedgeLabel(ring: Int, mode: WheelMode, chosen: String?, onTap: () -> Unit) {
    val wedge = wheelWedge(ring, mode) ?: return
    val major = mode == WheelMode.MAJOR
    val (x, y) =
        pointOn(if (major) (MAJOR_OUTER + RING_SPLIT) / 2 else (RING_SPLIT + MINOR_INNER) / 2, ring)
    val selected = chosen != null
    val ink = if (selected) IntradaColor.onAccent else IntradaColor.ink
    val quiet = if (selected) IntradaColor.onAccent else IntradaColor.inkSecondary
    val spoken = wedgeSpoken(ring, mode)
    Column(
        Modifier.offset(x - LABEL_BOX / 2, y - LABEL_BOX / 2)
            .size(LABEL_BOX)
            .testTag("itemForm.key.${mode.word}.$ring")
            .clearAndSetSemantics {
                contentDescription = spoken
                role = Role.Button
                this.selected = selected
                onClick {
                    onTap()
                    true
                }
            },
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        val alt = wedge.alt
        when {
            major && alt != null -> {
                val (top, bottom) =
                    if (chosen == alt) alt to wedge.primary else wedge.primary to alt
                BasicText(prettify(top), style = IntradaFont.segment.copy(color = ink))
                BasicText(
                    "⇅ ${prettify(bottom)}",
                    style = IntradaFont.small.copy(color = quiet),
                )
            }
            major ->
                BasicText(prettify(wedge.primary), style = IntradaFont.segment.copy(color = ink))
            else -> {
                if (alt != null) {
                    BasicText("⇅", style = IntradaFont.small.copy(color = quiet))
                }
                BasicText(
                    "${prettify(chosen ?: wedge.primary)}m",
                    style = IntradaFont.secondary.copy(color = quiet),
                )
            }
        }
    }
}
