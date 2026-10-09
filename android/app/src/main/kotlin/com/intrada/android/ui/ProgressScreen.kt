package com.intrada.android.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.intrada.android.R
import com.intrada.android.core.Store
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.scaled
import com.intrada.shared.AnalyticsView
import com.intrada.shared.Event
import com.intrada.shared.SessionEvent

@Composable
fun ProgressRoute(store: Store, onBuild: () -> Unit, modifier: Modifier = Modifier) {
    val viewModel by store.viewModel.collectAsState()
    val topMark = viewModel?.limits?.scoreMax?.toInt() ?: return
    ProgressScreen(
        viewModel?.analytics,
        topMark,
        onBuild = {
            val building = viewModel?.buildingSetlist != null
            if (building || store.sendAccepted(Event.Session(SessionEvent.StartBuilding))) onBuild()
        },
        modifier = modifier,
    )
}

@Composable
fun ProgressScreen(
    analytics: AnalyticsView?,
    topMark: Int,
    onBuild: () -> Unit,
    modifier: Modifier = Modifier,
) {
    ScreenScaffold("Progress", modifier, subtitle = analytics?.weekLine ?: "No sessions yet") {
        if (analytics == null) ProgressEmpty(onBuild) else ProgressBody(analytics, topMark)
    }
}

@Composable
private fun ProgressBody(analytics: AnalyticsView, topMark: Int) {
    Column(
        Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(IntradaSpacing.card),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.section),
    ) {
        analytics.topMover?.let { mover ->
            MasteryDeltaToast(
                "Mastery up",
                mover.itemTitle,
                was = mover.previousScore?.toInt() ?: 0,
                now = mover.currentScore.toInt(),
            )
        }
        MasteryHeroCard(
            analytics.overallMastery,
            topMark,
            analytics.masteryChange,
            analytics.climbing,
        )
        MarkSections(analytics)
        Section("Last five weeks", trailing = analytics.bestWeek) {
            ConsistencyBars(analytics.consistencyWeeks)
        }
        if (analytics.scoreChanges.isNotEmpty()) {
            Section("Recent mastery") {
                analytics.scoreChanges.forEach { change ->
                    MasteryDelta(
                        change.itemTitle,
                        if (change.isNew) "first time marked" else null,
                        was = change.previousScore?.toInt(),
                        now = change.currentScore.toInt(),
                    )
                }
            }
        }
    }
}

@Composable
private fun MarkSections(analytics: AnalyticsView) {
    if (analytics.variationCoverage.isNotEmpty()) {
        Section("Variations", trailing = analytics.variationCoverageCaption) {
            analytics.variationCoverage.forEach { row ->
                SolidCountRow(
                    row.title,
                    row.caption,
                    row.solid.toInt(),
                    row.total.toInt(),
                    row.spoken,
                )
            }
        }
    }
    listOf(
            "Variations across the library" to analytics.pooledVariations,
            "Keys across the library" to analytics.pooledKeys,
        )
        .filter { (_, rows) -> rows.isNotEmpty() }
        .forEach { (title, rows) ->
            Section(title) {
                rows.forEach { row ->
                    SolidCountRow(
                        row.label,
                        row.caption,
                        row.solid.toInt(),
                        row.total.toInt(),
                        "${row.label}, ${row.caption}",
                    )
                }
            }
        }
}

@Composable
private fun Section(
    title: String,
    trailing: String? = null,
    content: @Composable () -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact)) {
        if (trailing == null) FieldLabel(title) else SectionHeader(title, trailing = trailing)
        content()
    }
}

@Composable
private fun ProgressEmpty(onBuild: () -> Unit) {
    val message = "Minutes and marks will show here, week by week."
    Column(
        Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement =
            Arrangement.spacedBy(IntradaSpacing.section, Alignment.CenterVertically),
    ) {
        Column(
            Modifier.clearAndSetSemantics { contentDescription = message },
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
        ) {
            Image(
                painterResource(R.drawable.ic_tab_progress),
                contentDescription = null,
                modifier = Modifier.size(IntradaIconSize.hero.scaled()),
                colorFilter =
                    ColorFilter.tint(IntradaColor.accent.copy(alpha = IntradaOpacity.dimmed)),
            )
            BasicText(
                message,
                style =
                    IntradaFont.body.copy(
                        color = IntradaColor.inkSecondary,
                        textAlign = TextAlign.Center,
                    ),
            )
        }
        Box(
            Modifier.fillMaxWidth()
                .heightIn(min = 48.dp)
                .background(IntradaColor.marker, RoundedCornerShape(IntradaRadius.card))
                .clickable(role = Role.Button, onClick = onBuild)
                .testTag("progress.empty.build")
                .padding(vertical = IntradaSpacing.card),
            contentAlignment = Alignment.Center,
        ) {
            BasicText(
                "Build a session",
                style = IntradaFont.button.copy(color = IntradaColor.onMarker),
            )
        }
    }
}
