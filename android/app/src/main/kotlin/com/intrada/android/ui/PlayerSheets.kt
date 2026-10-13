package com.intrada.android.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.compose.ui.window.DialogWindowProvider
import androidx.core.view.WindowCompat
import com.intrada.android.R
import com.intrada.android.core.Store
import com.intrada.android.ui.components.FormErrorBanner
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface
import com.intrada.shared.BarRange
import com.intrada.shared.Event
import com.intrada.shared.PickerVariationView
import com.intrada.shared.SessionEvent
import com.intrada.shared.TempoReading

@Composable
internal fun PlayerSheet(
    title: String,
    onClose: () -> Unit,
    closeTag: String,
    content: @Composable () -> Unit,
) {
    Dialog(
        onDismissRequest = onClose,
        properties =
            DialogProperties(usePlatformDefaultWidth = false, decorFitsSystemWindows = false),
    ) {
        val view = LocalView.current
        SideEffect {
            (view.parent as? DialogWindowProvider)?.window?.let {
                WindowCompat.getInsetsController(it, view).apply {
                    isAppearanceLightStatusBars = true
                    isAppearanceLightNavigationBars = true
                }
            }
        }
        SheetPage(title, onClose, closeTag, content = content)
    }
}

@Composable
internal fun SheetPage(
    title: String,
    onClose: () -> Unit,
    closeTag: String,
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit,
) {
    ScreenScaffold(
        title,
        modifier,
        actions = {
            IconAction(R.drawable.ic_close, "Cancel", onClose, Modifier.testTag(closeTag))
        },
    ) {
        Column(Modifier.fillMaxSize().imePadding().verticalScroll(rememberScrollState())) {
            content()
        }
    }
}

// ── Switch variation (#1739) ──

class VariationChoice(
    val variations: List<PickerVariationView>,
    val currentId: String?,
)

/** [onPick] is false when the core refused, which keeps the page up beside the banner. */
@Composable
internal fun VariationPickerContent(choice: VariationChoice, onPick: (String) -> Boolean) {
    Column(Modifier.padding(top = IntradaSpacing.controlGap)) {
        choice.variations.forEachIndexed { index, variation ->
            if (index > 0) HairlineDivider(Modifier.padding(horizontal = IntradaSpacing.card))
            TickRow(
                variation.label,
                variation.caption.takeIf { it.isNotEmpty() },
                variation.id == choice.currentId,
                "variationPicker.row",
                { onPick(variation.id) },
            )
        }
        BasicText(
            "Switching starts a fresh count of repetitions.",
            Modifier.padding(IntradaSpacing.card),
            style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
        )
    }
}

// ── Trouble spot (#2249) ──

/** Bars as typed; the core places the spot among the item's sections and refuses what it cannot. */
@Composable
internal fun TroubleSpotContent(
    context: String,
    refusal: String?,
    onAdd: (first: Int, last: Int) -> Unit,
) {
    var first by remember { mutableStateOf("1") }
    var last by remember { mutableStateOf("4") }
    val bars = first.toIntOrNull()?.let { from -> last.toIntOrNull()?.let { from to it } }
    Column(
        Modifier.fillMaxWidth().padding(IntradaSpacing.card),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
    ) {
        BasicText(
            context,
            style =
                IntradaFont.secondary.copy(
                    color = IntradaColor.inkSecondary,
                    textAlign = TextAlign.Center,
                ),
        )
        Row(
            horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            BarField("First bar", first, { first = it }, "troubleSpot.first", Modifier.weight(1f))
            BasicText("to", style = IntradaFont.body.copy(color = IntradaColor.inkSecondary))
            BarField("Last bar", last, { last = it }, "troubleSpot.last", Modifier.weight(1f))
        }
        refusal?.let { FormErrorBanner(it, Modifier.fillMaxWidth()) }
        Box(Modifier.fillMaxWidth()) {
            if (bars != null) {
                InkButton("Add spot", "troubleSpot.add", { onAdd(bars.first, bars.second) })
            } else {
                BasicText(
                    "Enter both bar numbers",
                    Modifier.align(Alignment.Center),
                    style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
                )
            }
        }
    }
}

@Composable
private fun BarField(
    label: String,
    value: String,
    onChange: (String) -> Unit,
    tag: String,
    modifier: Modifier = Modifier,
) {
    FormField(
        label,
        value,
        { typed -> onChange(typed.filter(Char::isDigit)) },
        tag,
        modifier.cardSurface(IntradaRadius.control),
        keyboard = KeyboardType.Number,
    )
}

enum class RecordSheet {
    VARIATION,
    TROUBLE_SPOT,
}

@Composable
internal fun RecordSheets(
    sheet: RecordSheet,
    model: PlayerModel,
    reading: () -> TempoReading,
    send: (Event) -> Boolean,
    onClose: () -> Unit,
) {
    val active = model.active
    val entry = active.entries.getOrNull(active.currentPosition.toInt()) ?: return
    var refusal by remember(sheet) { mutableStateOf<String?>(null) }
    // Cleared from the core, or it would show again on the player once the page closes (#2009).
    val sendOrRefuse = { event: SessionEvent, fallback: String ->
        val accepted = send(Event.Session(event))
        if (accepted) {
            onClose()
        } else {
            val now = model.alertsNow()
            refusal = if (now.halted) Store.HALTED_MESSAGE else now.error ?: fallback
            send(Event.ClearError)
        }
        accepted
    }
    when (sheet) {
        RecordSheet.VARIATION ->
            PlayerSheet("Switch variation", onClose, "variationPicker.cancel") {
                refusal?.let { FormErrorBanner(it, Modifier.padding(IntradaSpacing.card)) }
                VariationPickerContent(
                    VariationChoice(
                        active.currentVariations,
                        active.currentVariationIds.firstOrNull(),
                    )
                ) { id ->
                    sendOrRefuse(
                        SessionEvent.SwitchPlay(
                            entry.id,
                            active.currentSectionId,
                            active.currentKey,
                            listOf(id),
                            SessionClock.now(),
                            reading(),
                        ),
                        "Couldn't switch. Try again.",
                    )
                }
            }
        RecordSheet.TROUBLE_SPOT ->
            PlayerSheet("Trouble spot", onClose, "troubleSpot.cancel") {
                val segment = active.record.segment
                TroubleSpotContent(
                    segment?.let { "${active.currentItemTitle} · in ${it.label}" }
                        ?: active.currentItemTitle,
                    refusal,
                ) { first, last ->
                    sendOrRefuse(
                        SessionEvent.AddTroubleSpot(entry.itemId, BarRange(first.bar, last.bar)),
                        "Couldn't add the spot. Try again.",
                    )
                }
            }
    }
}

private val Int.bar: UShort
    get() = coerceAtMost(UShort.MAX_VALUE.toInt()).toUShort()
