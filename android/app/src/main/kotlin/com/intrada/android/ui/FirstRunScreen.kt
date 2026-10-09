package com.intrada.android.ui

import androidx.activity.compose.BackHandler
import androidx.annotation.DrawableRes
import androidx.compose.foundation.Image
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
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.intrada.android.core.Store
import com.intrada.android.ui.components.FormErrorBanner
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.ProfileBadge
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.components.scaled
import com.intrada.shared.Event
import com.intrada.shared.FirstRunEvent
import com.intrada.shared.HighlighterColour
import com.intrada.shared.ItemKind
import com.intrada.shared.ProfileView

private enum class FirstRunStep {
    WELCOME,
    PROFILE,
    FIRST_PIECE,
}

private val READABLE_WIDTH = 560.dp

// The core decides whether the welcome is due; which step is up is screen state, as on iOS
// (#2117).
@Composable
fun FirstRunRoute(store: Store, onFinish: (added: Boolean) -> Unit, modifier: Modifier = Modifier) {
    var step by rememberSaveable { mutableStateOf(FirstRunStep.WELCOME) }
    var adding by rememberSaveable { mutableStateOf<ItemKind?>(null) }
    val form =
        rememberSaveable(saver = ProfileEditState.Saver) {
            ProfileEditState("", "", null, HighlighterColour.BUTTER)
        }
    val viewModel by store.viewModel.collectAsState()
    val profile = viewModel?.profile ?: return
    BackHandler(enabled = adding == null && step != FirstRunStep.WELCOME) {
        step = FirstRunStep.entries[step.ordinal - 1]
    }
    adding?.let { kind ->
        LibraryAddRoute(
            store,
            onDone = {
                adding = null
                if (store.viewModel.value?.firstRun?.added == true) onFinish(true)
            },
            modifier,
            kind,
        )
        return
    }
    when (step) {
        FirstRunStep.WELCOME ->
            WelcomeStep(
                onSetUpProfile = { step = FirstRunStep.PROFILE },
                onSkip = {
                    store.send(Event.FirstRun(FirstRunEvent.SkipWelcome))
                    onFinish(false)
                },
                modifier,
            )
        FirstRunStep.PROFILE ->
            ProfileStep(
                form,
                profile,
                onSave = {
                    form.refusal = store.saveProfile(form.profile())
                    if (form.refusal == null) step = FirstRunStep.FIRST_PIECE
                },
                onSkip = {
                    store.send(Event.FirstRun(FirstRunEvent.SkipWelcome))
                    step = FirstRunStep.FIRST_PIECE
                },
                modifier,
            )
        FirstRunStep.FIRST_PIECE ->
            FirstPieceStep(onAdd = { adding = it }, onSkip = { onFinish(false) }, modifier)
    }
}

@Composable
private fun StepFrame(modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    Box(
        modifier
            .fillMaxSize()
            .background(IntradaGradient.paper)
            .windowInsetsPadding(WindowInsets.safeDrawing),
        contentAlignment = Alignment.TopCenter,
    ) {
        Column(
            Modifier.widthIn(max = READABLE_WIDTH)
                .fillMaxSize()
                .padding(horizontal = IntradaSpacing.card),
            content = content,
        )
    }
}

// ── Welcome ──

@Composable
fun WelcomeStep(onSetUpProfile: () -> Unit, onSkip: () -> Unit, modifier: Modifier = Modifier) {
    StepFrame(modifier) {
        Row(Modifier.fillMaxWidth()) {
            Spacer(Modifier.weight(1f))
            SkipButton("firstRun.skipWelcome", onSkip)
        }
        Column(
            Modifier.weight(1f)
                .verticalScroll(rememberScrollState())
                .padding(top = IntradaSpacing.section),
            verticalArrangement = Arrangement.spacedBy(IntradaSpacing.section),
        ) {
            Column(
                Modifier.fillMaxWidth(),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
            ) {
                BasicText(
                    "Intrada",
                    Modifier.semantics { heading() },
                    style = IntradaFont.pageTitle.copy(color = IntradaColor.ink),
                )
                BasicText(
                    "A notebook for your practice.",
                    style =
                        IntradaFont.bodyMedium.copy(
                            color = IntradaColor.ink,
                            textAlign = TextAlign.Center,
                        ),
                )
            }
            Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.card)) {
                PillarLine(AppTab.LIBRARY, "The pieces and exercises you're working on.")
                PillarLine(AppTab.PRACTICE, "Build a session, play it through, mark how it went.")
                PillarLine(AppTab.PROGRESS, "What you've practised, week by week.")
            }
        }
        MarkerButton(
            "Set up profile",
            "firstRun.setUpProfile",
            IntradaColor.marker,
            onSetUpProfile,
            Modifier.padding(vertical = IntradaSpacing.card),
        )
    }
}

@Composable
private fun PillarLine(tab: AppTab, line: String) {
    Row(
        Modifier.semantics(mergeDescendants = true) {},
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
    ) {
        Box(Modifier.size(28.dp), contentAlignment = Alignment.Center) {
            StepIcon(tab.icon, IntradaColor.inkFaintIcon)
        }
        Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
            BasicText(tab.label, style = IntradaFont.cardTitle.copy(color = IntradaColor.ink))
            BasicText(line, style = IntradaFont.body.copy(color = IntradaColor.inkSecondary))
        }
    }
}

// ── Profile ──

@Composable
fun ProfileStep(
    form: ProfileEditState,
    profile: ProfileView,
    onSave: () -> Unit,
    onSkip: () -> Unit,
    modifier: Modifier = Modifier,
) {
    StepFrame(modifier) {
        form.refusal?.let {
            FormErrorBanner(
                it.message,
                Modifier.padding(top = IntradaSpacing.cardCompact).testTag("firstRun.profileError"),
            )
        }
        Column(
            Modifier.weight(1f)
                .verticalScroll(rememberScrollState())
                .padding(top = IntradaSpacing.section),
            verticalArrangement = Arrangement.spacedBy(IntradaSpacing.section),
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
            ) {
                StepTitle("Your profile", Modifier.weight(1f))
                ProfileBadge(profile.icon, IntradaColor.marker(form.colour))
            }
            Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.card)) {
                BasicText(
                    "All optional, and you can change it later.",
                    style = IntradaFont.body.copy(color = IntradaColor.inkSecondary),
                )
                ProfileNameFields(form, profile.instrumentNames)
                HighlighterSwatches(form.colour, { form.colour = it })
            }
        }
        Column(
            Modifier.padding(vertical = IntradaSpacing.card),
            verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
        ) {
            MarkerButton(
                "Save profile",
                "firstRun.saveProfile",
                IntradaColor.marker(form.colour),
                onSave,
            )
            SkipButton("firstRun.skipProfile", onSkip, Modifier.fillMaxWidth())
        }
    }
}

// ── First piece ──

@Composable
fun FirstPieceStep(onAdd: (ItemKind) -> Unit, onSkip: () -> Unit, modifier: Modifier = Modifier) {
    StepFrame(modifier) {
        Column(
            Modifier.weight(1f)
                .verticalScroll(rememberScrollState())
                .padding(top = IntradaSpacing.section),
            verticalArrangement = Arrangement.spacedBy(IntradaSpacing.section),
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact)) {
                StepTitle("Your first piece")
                BasicText(
                    "Add something you're working on.",
                    style = IntradaFont.body.copy(color = IntradaColor.inkSecondary),
                )
            }
            Column(Modifier.cardSurface()) {
                FirstPieceRow(
                    "Add a piece",
                    "Fill in the title and composer yourself",
                    "firstRun.typePiece",
                ) {
                    onAdd(ItemKind.PIECE)
                }
                HairlineDivider()
                FirstPieceRow(
                    "Add an exercise",
                    "A scale, a study, anything you repeat",
                    "firstRun.addExercise",
                ) {
                    onAdd(ItemKind.EXERCISE)
                }
            }
        }
        SkipButton(
            "firstRun.skipFirstPiece",
            onSkip,
            Modifier.fillMaxWidth().padding(vertical = IntradaSpacing.card),
        )
    }
}

@Composable
private fun FirstPieceRow(title: String, line: String, tag: String, onClick: () -> Unit) {
    Row(
        Modifier.fillMaxWidth()
            .clickable(role = Role.Button, onClick = onClick)
            .semantics(mergeDescendants = true) {}
            .testTag(tag)
            .padding(IntradaSpacing.card),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
    ) {
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            BasicText(title, style = IntradaFont.bodyMedium.copy(color = IntradaColor.ink))
            BasicText(line, style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary))
        }
        Chevron()
    }
}

// ── Parts ──

@Composable
private fun StepTitle(title: String, modifier: Modifier = Modifier) {
    BasicText(
        title,
        modifier.semantics { heading() },
        style = IntradaFont.pageTitle.copy(color = IntradaColor.ink),
    )
}

@Composable
private fun StepIcon(@DrawableRes icon: Int, tint: Color) {
    Image(
        painterResource(icon),
        contentDescription = null,
        modifier = Modifier.size(IntradaIconSize.control.scaled()),
        colorFilter = ColorFilter.tint(tint),
    )
}

@Composable
private fun MarkerButton(
    text: String,
    tag: String,
    marker: Color,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier
            .fillMaxWidth()
            .heightIn(min = 48.dp)
            .clip(RoundedCornerShape(IntradaRadius.card))
            .background(marker)
            .clickable(role = Role.Button, onClick = onClick)
            .testTag(tag)
            .padding(vertical = IntradaSpacing.card),
        contentAlignment = Alignment.Center,
    ) {
        BasicText(text, style = IntradaFont.button.copy(color = IntradaColor.onMarker))
    }
}

@Composable
private fun SkipButton(tag: String, onClick: () -> Unit, modifier: Modifier = Modifier) {
    Box(
        modifier
            .heightIn(min = 48.dp)
            .clickable(role = Role.Button, onClick = onClick)
            .testTag(tag)
            .padding(horizontal = IntradaSpacing.controlGap),
        contentAlignment = Alignment.Center,
    ) {
        BasicText("Skip", style = IntradaFont.bodyMedium.copy(color = IntradaColor.inkSecondary))
    }
}
