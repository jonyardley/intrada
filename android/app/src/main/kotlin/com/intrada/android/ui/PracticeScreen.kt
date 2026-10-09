package com.intrada.android.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.navigation.NavGraphBuilder
import androidx.navigation.NavHostController
import androidx.navigation.compose.composable
import com.intrada.android.core.Store
import com.intrada.android.core.resumeRecoverableSession
import com.intrada.shared.Event
import com.intrada.shared.HighlighterColour
import com.intrada.shared.LastPractisedView
import com.intrada.shared.PracticeSessionView
import com.intrada.shared.PracticeWeekView
import com.intrada.shared.SessionEvent
import kotlinx.coroutines.flow.drop

private const val SESSION_ROUTE = "practice/session"

fun NavGraphBuilder.practiceRoutes(
    store: Store,
    navController: NavHostController,
    onBuild: () -> Unit,
) {
    composable(AppTab.PRACTICE.route) {
        PracticeRoute(
            store,
            onBuild = onBuild,
            onOpen = { id -> navController.navigate("$SESSION_ROUTE/$id") },
        )
    }
    composable("$SESSION_ROUTE/{id}") { backStack ->
        SessionDetailRoute(store, backStack.arguments?.getString("id").orEmpty())
    }
}

@Composable
fun PracticeRoute(
    store: Store,
    onBuild: () -> Unit,
    modifier: Modifier = Modifier,
    onOpen: (String) -> Unit = {},
) {
    val viewModel by store.viewModel.collectAsState()
    val recoverable by store.recoverableSession.collectAsState()
    val weeks by store.practiceWeeks.collectAsState()
    val sessions by store.sessionHistory.collectAsState()
    PracticeScreen(
        PracticeModel(
            weeks,
            sessions,
            viewModel?.lastPractised,
            viewModel?.profile?.colour ?: HighlighterColour.BUTTER,
            viewModel?.profile?.greeting.orEmpty(),
        ),
        onStart = { if (store.sendAccepted(Event.Session(SessionEvent.StartBuilding))) onBuild() },
        onOpen = onOpen,
        modifier = modifier,
    ) {
        recoverable?.let { session ->
            RecoveryCard(
                session,
                onResume = { store.resumeRecoverableSession(SessionClock.now()) },
                onDiscard = store::discardSessionInProgress,
            )
        }
    }
}

class PracticeModel(
    val weeks: List<PracticeWeekView>,
    val sessions: List<PracticeSessionView>,
    val lastPractised: LastPractisedView?,
    val colour: HighlighterColour = HighlighterColour.BUTTER,
    val greeting: String = "",
)

@Composable
fun PracticeScreen(
    model: PracticeModel,
    onStart: () -> Unit,
    onOpen: (String) -> Unit,
    modifier: Modifier = Modifier,
    top: @Composable ColumnScope.() -> Unit = {},
) {
    ScreenScaffold("Practice", modifier, subtitle = model.subtitle) {
        Column(
            Modifier.fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(IntradaSpacing.card),
            verticalArrangement = Arrangement.spacedBy(IntradaSpacing.section),
        ) {
            top()
            LastPractisedHero(model.lastPractised, model.colour, onStart)
            History(model, onOpen)
        }
    }
}

// Null once there is a last-practised fact: the hero heading says it instead (#1725).
private val PracticeModel.subtitle: String?
    get() {
        val name = greeting.ifEmpty { null }
        if (lastPractised != null) return name
        return name?.let { "$it · No sessions yet" } ?: "No sessions yet"
    }

@Composable
private fun History(model: PracticeModel, onOpen: (String) -> Unit) {
    val weeks = model.weeks
    if (weeks.isEmpty()) return
    val pager = rememberPagerState(initialPage = weeks.lastIndex) { weeks.size }
    var selectedDay by rememberSaveable { mutableStateOf<String?>(null) }
    var knownWeeks by rememberSaveable { mutableIntStateOf(weeks.size) }
    LaunchedEffect(weeks.size) {
        if (weeks.size > knownWeeks && pager.currentPage == knownWeeks - 1) {
            pager.scrollToPage(weeks.lastIndex)
        }
        knownWeeks = weeks.size
    }
    LaunchedEffect(pager) {
        snapshotFlow { pager.currentPage }.drop(1).collect { selectedDay = null }
    }
    val week = weeks[pager.currentPage.coerceIn(0, weeks.lastIndex)]
    val day =
        week.days.firstOrNull { it.date == selectedDay }
            ?: week.days.getOrNull(week.openingDay.toInt())
            ?: week.days.lastOrNull()
    val daySessions =
        day?.sessionIds.orEmpty().mapNotNull { id -> model.sessions.firstOrNull { it.id == id } }
    val practised = week.practisedDays.toInt()

    val count = daySessions.size
    val dayTrailing =
        when {
            count > 0 -> "$count session${if (count == 1) "" else "s"}"
            day?.isFuture == true -> "Yet to come"
            else -> "Rest day"
        }

    Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.section)) {
        Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact)) {
            SectionHeader(
                "This week",
                trailing = "$practised day${if (practised == 1) "" else "s"} practised",
            )
            HorizontalPager(
                pager,
                Modifier.testTag("practice.weeks"),
                verticalAlignment = Alignment.Top,
            ) { page ->
                WeekStrip(weeks[page].days, day?.date) { selectedDay = it }
            }
        }
        Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact)) {
            SectionHeader(day?.heading.orEmpty(), trailing = dayTrailing)
            if (daySessions.isEmpty()) EmptyDayCard(future = day?.isFuture == true)
            daySessions.forEach { session -> SessionCard(session) { onOpen(session.id) } }
        }
    }
}
