package com.intrada.android.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.disabled
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.testTag
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import com.intrada.android.core.Store
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface
import com.intrada.shared.BuildingSetlistView
import com.intrada.shared.Event
import com.intrada.shared.LimitsView
import com.intrada.shared.SessionEvent
import com.intrada.shared.SetlistBlockView

// ── Practice tab ──

@Composable
fun PracticeRoute(store: Store, onBuild: () -> Unit, modifier: Modifier = Modifier) {
    val viewModel by store.viewModel.collectAsState()
    val building = viewModel?.buildingSetlist != null
    ScreenScaffold("Practice", modifier) {
        Column(Modifier.fillMaxSize().padding(IntradaSpacing.card)) {
            AddRow(
                if (building) "Carry on building" else "Build a session",
                if (building) "Carry on building the session" else "Build a session",
                "practice.build",
                {
                    if (building || store.sendAccepted(Event.Session(SessionEvent.StartBuilding)))
                        onBuild()
                },
                Modifier.cardSurface(),
                hint = "Pick pieces and exercises from the library",
            )
        }
    }
}

// ── Builder ──

class BuilderNavigation(
    val onAddItems: () -> Unit,
    val onAddExercise: (groupId: String) -> Unit,
    val onEntry: (entryId: String) -> Unit,
    val onClosed: () -> Unit,
)

class BuilderActions(
    val send: (Event) -> Boolean,
    val navigation: BuilderNavigation,
    val onDismissError: () -> Unit,
)

class BuilderModel(
    val setlist: BuildingSetlistView,
    val limits: LimitsView,
    val error: String? = null,
    val halted: Boolean = false,
)

class BuilderScreenState(editing: Boolean = false) {
    var editing by mutableStateOf(editing)
    var collapsed by mutableStateOf(emptySet<String>())
    var confirmingCancel by mutableStateOf(false)
    var menuFor by mutableStateOf<SetlistBlockView?>(null)
}

@Composable
fun BuilderRoute(store: Store, navigation: BuilderNavigation, modifier: Modifier = Modifier) {
    val viewModel by store.viewModel.collectAsState()
    val halted by store.halted.collectAsState()
    val state = remember { BuilderScreenState() }
    val setlist = viewModel?.buildingSetlist
    val limits = viewModel?.limits
    if (setlist == null || limits == null) {
        // Cancel or Start moves the core out of Building, and that is what closes the screen.
        SideEffect { navigation.onClosed() }
        return
    }
    BuilderScreen(
        BuilderModel(setlist, limits, viewModel?.error, halted),
        state,
        BuilderActions(store::sendAccepted, navigation) { store.send(Event.ClearError) },
        modifier,
    )
}

@Composable
fun BuilderScreen(
    model: BuilderModel,
    state: BuilderScreenState,
    actions: BuilderActions,
    modifier: Modifier = Modifier,
) {
    val setlist = model.setlist
    val cancel = {
        if (setlist.entries.isEmpty()) actions.send(Event.Session(SessionEvent.CancelBuilding))
        else state.confirmingCancel = true
    }
    BackHandler(onBack = { cancel() })
    if (setlist.blocks.isEmpty() && state.editing) state.editing = false
    ScreenScaffold(
        "Build session",
        modifier,
        actions = {
            TextAction("Cancel", "builder.cancel", { cancel() })
            if (setlist.blocks.isNotEmpty()) {
                TextAction(
                    if (state.editing) "Done" else "Edit",
                    "builder.edit",
                    { state.editing = !state.editing },
                    emphasised = state.editing,
                )
            }
        },
    ) {
        Column(Modifier.fillMaxSize()) {
            if (model.halted) GlobalBanner(Store.HALTED_MESSAGE, tag = "banner.halted")
            model.error?.let {
                GlobalBanner(it, tag = "banner.error", onDismiss = actions.onDismissError)
            }
            BuilderBody(model, state, actions)
            if (setlist.entries.isNotEmpty()) StartBar(setlist)
        }
    }
    state.menuFor?.let { BlockMenu(it, state, actions) }
    if (state.confirmingCancel) DiscardDialog(state, actions)
}

@Composable
private fun ColumnScope.BuilderBody(
    model: BuilderModel,
    state: BuilderScreenState,
    actions: BuilderActions,
) {
    val setlist = model.setlist
    Column(
        Modifier.weight(1f)
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(IntradaSpacing.card),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
    ) {
        BasicText(
            if (state.editing) "Editing" else summary(setlist),
            Modifier.testTag("builder.summary"),
            style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
        )
        if (!state.editing) LengthCard(setlist, model.limits, actions)
        if (setlist.blocks.isEmpty()) {
            BasicText(
                "Add pieces and exercises to build the session.",
                Modifier.padding(vertical = IntradaSpacing.card),
                style = IntradaFont.body.copy(color = IntradaColor.inkSecondary),
            )
        } else {
            if (setlist.blocks.any { it.groupId != null } && !state.editing) {
                Box(Modifier.fillMaxWidth(), contentAlignment = Alignment.CenterEnd) {
                    TextAction(
                        "Ungroup all",
                        "builder.ungroupAll",
                        { actions.send(Event.Session(SessionEvent.UngroupAllBlocks)) },
                    )
                }
            }
            BuilderList(setlist, state, actions)
        }
        AddRow(
            "Add piece or exercise",
            "Add a piece or exercise",
            "builder.addItems",
            actions.navigation.onAddItems,
            Modifier.cardSurface(),
        )
    }
}

@Composable
private fun LengthCard(setlist: BuildingSetlistView, limits: LimitsView, actions: BuilderActions) {
    val length = setlist.lengthMins?.toInt()
    val step = limits.sessionLengthStepMins.toInt()
    val send = { mins: UShort? -> actions.send(Event.Session(SessionEvent.SetSessionLength(mins))) }
    Column(
        Modifier.fillMaxWidth()
            .cardSurface(IntradaRadius.control)
            .padding(horizontal = IntradaSpacing.card, vertical = IntradaSpacing.controlGap)
    ) {
        ToggleRow(
            "Today's length",
            on = length != null,
            tag = "builder.sessionLength.toggle",
            onChange = { on -> send(if (on) limits.sessionLengthDefaultMins else null) },
        )
        if (length != null) {
            StepperRow(
                setlist.lengthSummary.orEmpty(),
                "time today",
                "builder.sessionLength.stepper",
                onStep = { send((length + it * step).toUShort()) },
                canDecrease = length - step >= limits.sessionLengthMinMins.toInt(),
                canIncrease = length + step <= limits.sessionLengthMaxMins.toInt(),
            )
        }
    }
}

// Start hands over to the player, C2 of specs/android-shell.md (#2422); until then it waits.
@Composable
private fun StartBar(setlist: BuildingSetlistView) {
    Column(Modifier.fillMaxWidth().background(IntradaColor.paperTop)) {
        HairlineDivider()
        Box(
            Modifier.fillMaxWidth()
                .padding(horizontal = IntradaSpacing.card, vertical = IntradaSpacing.cardCompact)
                .heightIn(min = 48.dp)
                .clip(RoundedCornerShape(IntradaRadius.control))
                .background(IntradaColor.surfaceSunken)
                .clearAndSetSemantics {
                    contentDescription = "Start session, not on Android yet"
                    testTag = "builder.start"
                    role = Role.Button
                    disabled()
                },
            contentAlignment = Alignment.Center,
        ) {
            BasicText(
                startTitle(setlist),
                style = IntradaFont.button.copy(color = IntradaColor.inkSecondary),
            )
        }
    }
}

@Composable
private fun BlockMenu(block: SetlistBlockView, state: BuilderScreenState, actions: BuilderActions) {
    val groupId = block.groupId ?: return
    val close = { state.menuFor = null }
    val choose = { event: SessionEvent ->
        close()
        actions.send(Event.Session(event))
    }
    Dialog(onDismissRequest = close) {
        Column(Modifier.fillMaxWidth().cardSurface()) {
            FieldLabel(
                block.pieceTitle ?: "Block",
                Modifier.padding(
                    horizontal = IntradaSpacing.card,
                    vertical = IntradaSpacing.cardCompact,
                ),
            )
            block.piece?.let { piece ->
                MenuRow("Piece settings", "builder.menu.settings") {
                    close()
                    actions.navigation.onEntry(piece.id)
                }
            }
            MenuRow("Just the piece", "builder.menu.pieceOnly") {
                choose(SessionEvent.KeepOnlyPiece(groupId))
            }
            MenuRow("Ungroup", "builder.menu.ungroup") {
                choose(SessionEvent.UngroupBlock(groupId))
            }
            MenuRow("Remove block", "builder.menu.remove", danger = true) {
                choose(SessionEvent.RemoveBlock(groupId))
            }
        }
    }
}

@Composable
private fun MenuRow(title: String, tag: String, danger: Boolean = false, onClick: () -> Unit) {
    Column {
        HairlineDivider()
        BasicText(
            title,
            Modifier.fillMaxWidth()
                .heightIn(min = 48.dp)
                .clickable(role = Role.Button, onClick = onClick)
                .testTag(tag)
                .padding(horizontal = IntradaSpacing.card, vertical = IntradaSpacing.cardCompact),
            style =
                IntradaFont.body.copy(
                    color = if (danger) IntradaColor.danger else IntradaColor.ink
                ),
        )
    }
}

@Composable
private fun DiscardDialog(state: BuilderScreenState, actions: BuilderActions) {
    ConfirmDialog(
        ConfirmCopy(
            "Discard session plan?",
            "The items you've added will be cleared.",
            "Discard",
            "builder.discard",
        ),
        onConfirm = {
            state.confirmingCancel = false
            actions.send(Event.Session(SessionEvent.CancelBuilding))
        },
        onDismiss = { state.confirmingCancel = false },
    )
}
