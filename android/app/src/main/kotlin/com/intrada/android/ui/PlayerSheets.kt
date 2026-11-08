package com.intrada.android.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
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
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.compose.ui.window.DialogWindowProvider
import androidx.core.view.WindowCompat
import com.intrada.android.R
import com.intrada.android.core.Store
import com.intrada.android.ui.components.FormErrorBanner
import com.intrada.android.ui.components.HairlineDivider
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

enum class RecordSheet {
    VARIATION
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
                            active.currentKey,
                            listOf(id),
                            SessionClock.now(),
                            reading(),
                        ),
                        "Couldn't switch. Try again.",
                    )
                }
            }
    }
}
