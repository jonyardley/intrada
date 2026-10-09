package com.intrada.android.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
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
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTag
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.navigation.NavGraphBuilder
import androidx.navigation.compose.composable
import com.intrada.android.core.Store
import com.intrada.android.ui.components.ProfileBadge
import com.intrada.android.ui.components.cardSurface
import com.intrada.shared.Event
import com.intrada.shared.HighlighterColour
import com.intrada.shared.InstrumentIcon
import com.intrada.shared.LimitsView
import com.intrada.shared.PracticeDefaults
import com.intrada.shared.PracticeDefaultsEvent
import com.intrada.shared.ProfileView

const val PROFILE_ROUTE = "practice/profile"

fun NavGraphBuilder.profileRoute(store: Store) {
    composable(PROFILE_ROUTE) { ProfileRoute(store) }
}

class ProfileActions(
    val onEdit: () -> Unit,
    val onDefaults: (PracticeDefaults) -> Unit,
    val onFeedback: () -> Unit,
)

@Composable
fun ProfileRoute(store: Store, modifier: Modifier = Modifier) {
    val viewModel by store.viewModel.collectAsState()
    val view = viewModel ?: return
    var editing by rememberSaveable { mutableStateOf(false) }
    var feedback by rememberSaveable { mutableStateOf(false) }
    when {
        editing -> {
            BackHandler { editing = false }
            ProfileEditRoute(store, view.profile, onDone = { editing = false }, modifier)
        }
        feedback -> FeedbackSheet(null, onDismiss = { feedback = false }, modifier)
        else ->
            ProfileScreen(
                view.profile,
                view.practiceDefaults,
                view.limits,
                ProfileActions(
                    onEdit = { editing = true },
                    onDefaults = {
                        store.send(Event.PracticeDefaults(PracticeDefaultsEvent.Save(it)))
                    },
                    onFeedback = { feedback = true },
                ),
                modifier,
                storeAlerts(store),
            )
    }
}

/** Who is practising: everything shown comes from the core's profile view (#1692). */
@Composable
fun ProfileScreen(
    profile: ProfileView,
    defaults: PracticeDefaults,
    limits: LimitsView,
    actions: ProfileActions,
    modifier: Modifier = Modifier,
    alerts: ScreenAlerts = ScreenAlerts(),
) {
    ScreenScaffold(
        "Profile",
        modifier,
        actions = { TextAction("Edit", "profile.edit", actions.onEdit) },
    ) {
        Column(Modifier.fillMaxSize()) {
            AlertBanners(alerts)
            Column(
                Modifier.weight(1f)
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
                    .padding(horizontal = IntradaSpacing.card)
                    .padding(bottom = IntradaSpacing.section),
                verticalArrangement = Arrangement.spacedBy(IntradaSpacing.section),
            ) {
                ProfileHero(profile)
                HighlighterCard(profile.colour)
                PracticeDefaultsSection(defaults, limits, actions.onDefaults)
                FeedbackRow(actions.onFeedback)
            }
        }
    }
}

@Composable
private fun ProfileHero(profile: ProfileView) {
    Column(
        Modifier.fillMaxWidth()
            .padding(top = IntradaSpacing.section, bottom = IntradaSpacing.controlGap)
            .semantics(mergeDescendants = true) {},
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
    ) {
        ProfileBadge(profile.icon, IntradaColor.marker(profile.colour), size = IntradaGlyph.hero)
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            val named = profile.name.isNotEmpty()
            BasicText(
                profile.name.ifEmpty { "Add a name" },
                style =
                    IntradaFont.pageTitle.copy(
                        color = if (named) IntradaColor.ink else IntradaColor.inkSecondary,
                        textAlign = TextAlign.Center,
                    ),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            BasicText(
                profile.instrument.ifEmpty { "Add an instrument" },
                style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
            )
        }
    }
}

@Composable
private fun HighlighterCard(colour: HighlighterColour) {
    Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact)) {
        SectionHeader("Highlighter")
        Row(
            Modifier.fillMaxWidth()
                .cardSurface()
                .clearAndSetSemantics {
                    contentDescription = "Highlighter, ${colour.label}"
                    testTag = "profile.highlighter"
                }
                .padding(horizontal = IntradaSpacing.card, vertical = IntradaSpacing.cardCompact),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
        ) {
            Box(Modifier.size(22.dp).background(IntradaColor.marker(colour), CircleShape))
            BasicText(colour.label, style = IntradaFont.bodyMedium.copy(color = IntradaColor.ink))
        }
    }
}

@Composable
private fun FeedbackRow(onFeedback: () -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact)) {
        SectionHeader("Beta")
        Row(
            Modifier.fillMaxWidth()
                .cardSurface()
                .heightIn(min = 48.dp)
                .clickable(role = Role.Button, onClick = onFeedback)
                .clearAndSetSemantics {
                    contentDescription = "Send feedback"
                    role = Role.Button
                    onClick("Opens the feedback form") {
                        onFeedback()
                        true
                    }
                }
                .testTag("profile.sendFeedback")
                .padding(horizontal = IntradaSpacing.card, vertical = IntradaSpacing.cardCompact),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            BasicText(
                "Send feedback",
                Modifier.weight(1f),
                style = IntradaFont.body.copy(color = IntradaColor.ink),
            )
            Chevron()
        }
    }
}

internal val HighlighterColour.label: String
    get() =
        when (this) {
            HighlighterColour.BUTTER -> "Butter"
            HighlighterColour.CORAL -> "Coral"
            HighlighterColour.MINT -> "Mint"
            HighlighterColour.SKY -> "Sky"
            HighlighterColour.LAVENDER -> "Lavender"
            HighlighterColour.SAGE -> "Sage"
            HighlighterColour.PEACH -> "Peach"
            HighlighterColour.POWDER -> "Powder"
        }

internal val InstrumentIcon.tileLabel: String
    get() =
        when (this) {
            InstrumentIcon.PIANO -> "Piano"
            InstrumentIcon.ACOUSTICGUITAR -> "Guitar"
            InstrumentIcon.ELECTRICGUITAR -> "Electric guitar"
            InstrumentIcon.VIOLIN -> "Violin"
            InstrumentIcon.CELLO -> "Cello"
            InstrumentIcon.VOICE -> "Voice"
            InstrumentIcon.FLUTE -> "Flute"
            InstrumentIcon.CLARINET -> "Clarinet"
            InstrumentIcon.SAXOPHONE -> "Saxophone"
            InstrumentIcon.TRUMPET -> "Trumpet"
            InstrumentIcon.DRUMS -> "Drums"
            InstrumentIcon.HARP -> "Harp"
            InstrumentIcon.OTHER -> "Other"
        }

@Composable
internal fun ProfileButton(profile: ProfileView, onOpen: () -> Unit) {
    Box(
        Modifier.size(48.dp).clickable(role = Role.Button, onClick = onOpen).clearAndSetSemantics {
            contentDescription = "Profile"
            role = Role.Button
            testTag = "practice.profile"
            onClick {
                onOpen()
                true
            }
        },
        contentAlignment = Alignment.Center,
    ) {
        ProfileBadge(profile.icon, IntradaColor.marker(profile.colour), size = IntradaGlyph.bar)
    }
}
