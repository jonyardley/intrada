package com.intrada.android.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import com.intrada.android.ui.components.cardSurface
import com.intrada.shared.ActiveSession
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle

@Composable
fun RecoveryCard(
    session: ActiveSession,
    onResume: () -> Unit,
    onDiscard: () -> Unit,
    modifier: Modifier = Modifier,
    today: Instant = Instant.now(),
    zone: ZoneId = ZoneId.systemDefault(),
) {
    Column(
        modifier.fillMaxWidth().cardSurface().padding(IntradaSpacing.card),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
    ) {
        BasicText(
            "Pick up where you left off?",
            style = IntradaFont.cardTitle.copy(color = IntradaColor.ink),
        )
        BasicText(
            recoveryMeta(session, today, zone),
            style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
        )
        Row(
            Modifier.fillMaxWidth().padding(top = IntradaSpacing.controlGap),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
        ) {
            InkButton(
                "Resume",
                "practice.resume",
                onResume,
                Modifier.weight(1f),
                spoken = "Resume the interrupted session",
            )
            TextAction(
                "Discard",
                "practice.discardResume",
                onDiscard,
                Modifier.semantics { contentDescription = "Discard the interrupted session" },
            )
        }
    }
}

/** An old practice saying just "09:02" would read as today, so another day shows its date. */
internal fun recoveryMeta(session: ActiveSession, today: Instant, zone: ZoneId): String {
    val total = session.entries.size
    val position = minOf(session.currentIndex.toInt() + 1, total)
    val count = "$position of $total items"
    val started = SessionClock.parse(session.sessionStartedAt) ?: return count
    val day = started.atZone(zone)
    val style =
        if (day.toLocalDate() == today.atZone(zone).toLocalDate())
            DateTimeFormatter.ofLocalizedTime(FormatStyle.SHORT)
        else DateTimeFormatter.ofLocalizedDateTime(FormatStyle.MEDIUM, FormatStyle.SHORT)
    return "$count · started ${day.format(style)}"
}
