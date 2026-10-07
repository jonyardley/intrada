package com.intrada.android.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LifecycleEventEffect
import com.intrada.android.core.Store
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.components.label
import com.intrada.shared.ActiveSessionView
import com.intrada.shared.Event
import com.intrada.shared.LimitsView
import com.intrada.shared.SessionEvent
import com.intrada.shared.SummaryView
import com.intrada.shared.TempoReading
import java.time.Instant

// ── The player, its reflection and the summary, over the tabs while a session is live ──

@Composable
fun PlayerHost(store: Store, modifier: Modifier = Modifier) {
    val viewModel by store.viewModel.collectAsState()
    val alerts = storeAlerts(store)
    val limits = viewModel?.limits ?: return
    val active = viewModel?.activeSession
    val summary = viewModel?.summary
    val send = store::sendAccepted
    when {
        active != null -> {
            KeepScreenOn()
            LifecycleEventEffect(Lifecycle.Event.ON_STOP) {
                store.send(Event.Session(SessionEvent.WentAway(SessionClock.now())))
            }
            LifecycleEventEffect(Lifecycle.Event.ON_START) {
                store.send(Event.Session(SessionEvent.CameBack(SessionClock.now())))
            }
            PlayerScreen(PlayerModel(active, limits, alerts), send, modifier)
        }
        summary != null -> SummaryScreen(SummaryModel(summary, limits, alerts), send, modifier)
    }
}

@Composable
private fun KeepScreenOn() {
    val view = LocalView.current
    DisposableEffect(view) {
        view.keepScreenOn = true
        onDispose { view.keepScreenOn = false }
    }
}

class PlayerModel(
    val active: ActiveSessionView,
    val limits: LimitsView,
    val alerts: ScreenAlerts = ScreenAlerts(),
    /** A fixed instant for tests; the app passes none and the clocks tick. */
    val held: Instant? = null,
)

class SummaryModel(
    val summary: SummaryView,
    val limits: LimitsView,
    val alerts: ScreenAlerts = ScreenAlerts(),
)

/** No click on Android yet (#2460), so every play is stamped as silent at the item's own tempo. */
internal val ActiveSessionView.reading: TempoReading
    get() = TempoReading(clickSeedBpm, clickSounding = false, click = null)

@Composable
fun PlayerScreen(model: PlayerModel, send: (Event) -> Boolean, modifier: Modifier = Modifier) {
    val active = model.active
    val reflection = active.reflection
    if (reflection != null) {
        ReflectionRoute(model, reflection, send, modifier)
        return
    }
    var options by remember { mutableStateOf(false) }
    var keptAway by remember { mutableStateOf<String?>(null) }
    BackHandler { options = true }
    val density = LocalDensity.current
    Column(
        modifier
            .fillMaxSize()
            .background(IntradaGradient.playerPaper(density))
            .windowInsetsPadding(WindowInsets.safeDrawing)
            .padding(horizontal = IntradaSpacing.card)
            .padding(top = IntradaSpacing.card)
    ) {
        val now = rememberTicking(model.held)
        val sessionStart = SessionClock.parse(active.startedAt)
        OrientationBand(
            sessionStart?.let { SessionClock.secondsBetween(it, now) },
            "Focus · ${active.currentPosition + 1u} of ${active.totalItems}",
            active.entries.map { it.itemType },
            minOf(active.currentPosition.toInt() + 1, active.totalItems.toInt()),
        ) {
            MenuButton("Session options", "player.options") { options = true }
        }
        AlertBanners(model.alerts)
        active.record.awayOffer
            ?.takeIf { it.label != keptAway }
            ?.let { offer ->
                AwayOffer(
                    offer.label,
                    onLeaveOut = { send(Event.Session(SessionEvent.LeaveAwayOut)) },
                    onKeep = { keptAway = offer.label },
                )
            }
        PlayerBody(active, now, send)
        Transport(active, send)
    }
    if (options) OptionsDialog(active, send, onDismiss = { options = false })
}

@Composable
private fun ColumnScope.PlayerBody(
    active: ActiveSessionView,
    now: Instant,
    send: (Event) -> Boolean,
) {
    val stamped = { event: (String, TempoReading) -> SessionEvent ->
        send(Event.Session(event(SessionClock.now(), active.reading)))
    }
    Column(
        Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Spacer(Modifier.heightIn(min = IntradaSpacing.card))
        CentreInfo(active)
        SessionClock.parse(active.currentItemStartedAt)?.let { itemStart ->
            TimerRing(
                SessionClock.secondsBetween(itemStart, now),
                active.currentPlannedDurationSecs?.toLong(),
                Modifier.padding(top = IntradaSpacing.section),
            )
        }
        RepCounter(
            RepState(
                count = active.currentRepCount?.toInt() ?: 0,
                slots = active.currentRepSlots.toInt(),
                touched = active.currentRepCount != null,
                reached = active.currentRepTargetReached ?: false,
                extra = active.currentRepsPastTarget.toInt(),
                canUndo = active.currentCanUndo,
            ),
            RepActions(
                onGotIt = { stamped(SessionEvent::RepGotIt) },
                onNotQuite = { stamped(SessionEvent::RepMissed) },
                onUndo = { stamped(SessionEvent::RepUndo) },
            ),
            Modifier.padding(top = IntradaSpacing.section),
        )
        Spacer(Modifier.heightIn(min = IntradaSpacing.card))
    }
}

@Composable
private fun CentreInfo(active: ActiveSessionView) {
    Column(
        Modifier.fillMaxWidth().padding(horizontal = IntradaSpacing.card),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
    ) {
        BasicText(
            active.currentItemType.label,
            style = IntradaFont.badge.copy(color = IntradaColor.inkSecondary),
        )
        BasicText(
            active.currentItemTitle,
            Modifier.testTag("player.title"),
            style =
                IntradaFont.pageTitle.copy(color = IntradaColor.ink, textAlign = TextAlign.Center),
        )
        active.currentRelatedPieceTitle?.let {
            BasicText(
                "Related to $it",
                style = IntradaFont.secondary.copy(color = IntradaColor.accent),
            )
        }
        active.currentPlayLabel?.let {
            BasicText(it, style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary))
        }
        active.currentItemIntention?.takeIf { it.isNotEmpty() }?.let { Secondary("Aim: $it") }
        active.currentItemNotes
            ?.takeIf { it.isNotEmpty() }
            ?.let { Secondary("Notes: $it", NOTES_LINES) }
    }
}

private const val NOTES_LINES = 3

@Composable
private fun Secondary(text: String, maxLines: Int = Int.MAX_VALUE) {
    BasicText(
        text,
        style =
            IntradaFont.secondary.copy(
                color = IntradaColor.inkSecondary,
                textAlign = TextAlign.Center,
            ),
        maxLines = maxLines,
        overflow = TextOverflow.Ellipsis,
    )
}

@Composable
private fun AwayOffer(label: String, onLeaveOut: () -> Unit, onKeep: () -> Unit) {
    Row(
        Modifier.fillMaxWidth()
            .padding(top = IntradaSpacing.controlGap)
            .cardSurface(IntradaRadius.control)
            .padding(start = IntradaSpacing.card),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        BasicText(
            label,
            Modifier.weight(1f),
            style = IntradaFont.secondary.copy(color = IntradaColor.ink),
        )
        TextAction("Leave it out", "player.leaveAwayOut", onLeaveOut)
        IconAction(com.intrada.android.R.drawable.ic_close, "Keep the time", onKeep)
    }
}

@Composable
private fun Transport(active: ActiveSessionView, send: (Event) -> Boolean) {
    Column(
        Modifier.fillMaxWidth().padding(bottom = IntradaSpacing.card),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
    ) {
        val last = active.nextItemTitle == null
        InkButton(
            if (last) "Finish session" else "Next item",
            "player.advance",
            {
                send(
                    Event.Session(
                        SessionEvent.PrepareReflection(SessionClock.now(), active.reading)
                    )
                )
            },
        )
        TextAction(
            "Skip this item",
            "player.skip",
            { send(Event.Session(SessionEvent.SkipItem(SessionClock.now()))) },
        )
        active.nextItemTitle?.let {
            BasicText(
                "Next · $it",
                style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Composable
private fun OptionsDialog(
    active: ActiveSessionView,
    send: (Event) -> Boolean,
    onDismiss: () -> Unit,
) {
    Dialog(onDismissRequest = onDismiss) {
        Column(Modifier.fillMaxWidth().cardSurface()) {
            OptionRow("Skip this item", "player.options.skip") {
                onDismiss()
                send(Event.Session(SessionEvent.SkipItem(SessionClock.now())))
            }
            HairlineDivider()
            OptionRow("End session early", "player.options.end", danger = true) {
                onDismiss()
                send(
                    Event.Session(SessionEvent.EndSessionEarly(SessionClock.now(), active.reading))
                )
            }
            HairlineDivider()
            OptionRow("Keep practising", "player.options.cancel", onClick = onDismiss)
        }
    }
}

@Composable
private fun OptionRow(title: String, tag: String, danger: Boolean = false, onClick: () -> Unit) {
    Box(
        Modifier.fillMaxWidth()
            .heightIn(min = 48.dp)
            .clickable(role = Role.Button, onClick = onClick)
            .testTag(tag)
            .padding(horizontal = IntradaSpacing.card, vertical = IntradaSpacing.cardCompact),
        contentAlignment = Alignment.CenterStart,
    ) {
        BasicText(
            title,
            style =
                IntradaFont.body.copy(
                    color = if (danger) IntradaColor.danger else IntradaColor.ink
                ),
        )
    }
}
