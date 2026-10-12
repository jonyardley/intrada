package com.intrada.android.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.intrada.android.ui.components.FormErrorBanner
import com.intrada.android.ui.components.TagChip
import com.intrada.android.ui.components.TagChipStyle
import com.intrada.android.ui.components.cardSurface
import com.intrada.ffi.WheelMode
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKeyView
import com.intrada.shared.Key
import com.intrada.shared.LibraryItemView

// ── Keys on the piece ──

@Composable
internal fun KeysCard(item: LibraryItemView, topMark: Int, onChoose: () -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact)) {
        SectionHeader(
            "Keys",
            caption = item.keysCaption,
            action = HeaderAction("Choose", "Choose keys", "keys.header", perform = onChoose),
        )
        Row(
            Modifier.fillMaxWidth()
                .cardSurface()
                .horizontalScroll(rememberScrollState())
                .padding(IntradaSpacing.cardCompact),
            horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
        ) {
            item.keys.forEach { KeyRingItem(it, topMark) }
        }
    }
}

@Composable
internal fun OtherKeysRow(onChoose: () -> Unit) {
    AddRow(
        "Practise in other keys",
        "Practise in other keys",
        "keys.choose",
        onChoose,
        Modifier.cardSurface(),
    )
}

@Composable
private fun KeyRingItem(key: ItemKeyView, topMark: Int) {
    Column(
        Modifier.clearAndSetSemantics { contentDescription = "${key.label}, ${key.caption}" },
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        KeyRing(key.latestScore?.toInt(), topMark, shortLabel(key))
        BasicText(
            key.caption,
            style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
        )
    }
}

private fun shortLabel(key: ItemKeyView): String {
    val selection = wheelSelection(key.key) ?: return key.label
    val tonic = prettify(selection.spelling)
    return if (selection.mode == WheelMode.MINOR) "${tonic}m" else tonic
}

@Composable
private fun KeyRing(score: Int?, topMark: Int, label: String, size: Dp = 44.dp) {
    val fraction =
        if (score == null || score <= 0 || topMark <= 0) 0f
        else score.coerceAtMost(topMark).toFloat() / topMark
    val line = maxOf(3.dp, size * RING_LINE)
    Box(Modifier.size(size), contentAlignment = Alignment.Center) {
        Canvas(Modifier.fillMaxSize()) {
            val stroke = line.toPx()
            val inset = stroke / 2
            val arc = Size(this.size.width - stroke, this.size.height - stroke)
            drawArc(
                IntradaColor.masteryTrack,
                0f,
                FULL_TURN,
                false,
                Offset(inset, inset),
                arc,
                style = Stroke(stroke),
            )
            if (fraction > 0f) {
                drawArc(
                    IntradaColor.masteryFill,
                    TWELVE_O_CLOCK,
                    FULL_TURN * fraction,
                    false,
                    Offset(inset, inset),
                    arc,
                    style = Stroke(stroke, cap = StrokeCap.Round),
                )
            }
        }
        BasicText(
            label,
            maxLines = 1,
            style =
                IntradaFont.scoreNumeral((size.value * LABEL).sp)
                    .copy(
                        color = if (fraction > 0f) IntradaColor.ink else IntradaColor.inkSecondary
                    ),
        )
    }
}

private const val RING_LINE = 0.09f
private const val LABEL = 0.33f
private const val FULL_TURN = 360f
private const val TWELVE_O_CLOCK = -90f

// ── Keys sheet ──

@Composable
internal fun KeysSheet(
    item: LibraryItemView,
    sendFromSheet: (Event) -> String?,
    onClose: () -> Unit,
) {
    FullScreenDialog(onClose) { KeysSheetScreen(item, sendFromSheet, onClose) }
}

@Composable
internal fun KeysSheetScreen(
    item: LibraryItemView,
    sendFromSheet: (Event) -> String?,
    onClose: () -> Unit,
) {
    var keys by remember { mutableStateOf(item.keys.map { it.key }) }
    var formError by remember { mutableStateOf<String?>(null) }
    ScreenScaffold(
        "Keys",
        actions = {
            TextAction("Cancel", "keysSheet.cancel", onClose)
            TextAction(
                "Done",
                "sheet.done",
                {
                    if (keys == item.keys.map { it.key }) {
                        onClose()
                    } else {
                        formError = sendFromSheet(Event.Item(ItemEvent.UpdateKeys(item.id, keys)))
                        if (formError == null) onClose()
                    }
                },
                emphasised = true,
            )
        },
    ) {
        Column(
            Modifier.fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(IntradaSpacing.card),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
        ) {
            formError?.let { FormErrorBanner(it, Modifier.testTag("keysSheet.error")) }
            KeysChooser(keys) { keys = it }
            BasicText(
                "Tap a key to add it. Tap a chosen key with ⇅ to switch its spelling; tap again to remove it.",
                style =
                    IntradaFont.small.copy(
                        color = IntradaColor.inkSecondary,
                        textAlign = TextAlign.Center,
                    ),
            )
        }
    }
}

@Composable
private fun KeysChooser(keys: List<Key>, onKeys: (List<Key>) -> Unit) {
    val haptics = LocalHapticFeedback.current
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
    ) {
        val lit = keys.mapNotNull(::wheelSelection)
        KeyWheel(
            chosen = { ring, mode ->
                lit.firstOrNull { it.ring.toInt() == ring && it.mode == mode }?.spelling
            },
            onTap = { ring, mode ->
                onKeys(keysAfterTap(keys, ring, mode))
                haptics.play(Haptic.SELECTION)
            },
            tag = "keysSheet.key",
        ) {
            Column(
                Modifier.semantics(mergeDescendants = true) {},
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                BasicText(
                    "${keys.size}",
                    style = IntradaFont.title.copy(color = IntradaColor.ink),
                )
                BasicText(
                    if (keys.size == 1) "key chosen" else "keys chosen",
                    style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
                )
            }
        }
        Row(horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap)) {
            PresetChip("All major", "keysSheet.allMajor") {
                onKeys(keysWithAll(keys, WheelMode.MAJOR))
                haptics.play(Haptic.IMPACT)
            }
            PresetChip("All minor", "keysSheet.allMinor") {
                onKeys(keysWithAll(keys, WheelMode.MINOR))
                haptics.play(Haptic.IMPACT)
            }
            PresetChip("Clear", "keysSheet.clear") {
                onKeys(emptyList())
                haptics.play(Haptic.IMPACT)
            }
        }
    }
}

@Composable
private fun PresetChip(title: String, tag: String, onClick: () -> Unit) {
    Box(
        Modifier.heightIn(min = 48.dp)
            .clickable(role = Role.Button, onClick = onClick)
            .testTag(tag),
        contentAlignment = Alignment.Center,
    ) {
        TagChip(title, style = TagChipStyle.Outlined)
    }
}
