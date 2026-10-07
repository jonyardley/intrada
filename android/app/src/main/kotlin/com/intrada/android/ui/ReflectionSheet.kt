package com.intrada.android.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.union
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LifecycleEventEffect
import com.intrada.android.core.Store
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.FormErrorBanner
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface
import com.intrada.shared.DraftMark
import com.intrada.shared.DraftTempo
import com.intrada.shared.Event
import com.intrada.shared.Felt
import com.intrada.shared.FinishSheetView
import com.intrada.shared.IntentionMet
import com.intrada.shared.LimitsView
import com.intrada.shared.Obstacle
import com.intrada.shared.PlayView
import com.intrada.shared.ReflectionAnswers
import com.intrada.shared.ReflectionView
import com.intrada.shared.SessionEvent
import kotlinx.coroutines.delay

/**
 * What the musician has answered so far. Every change goes to the core as a draft, so a resume
 * reopens the sheet as it was left (#2137).
 */
class ReflectionState(seed: ReflectionAnswers, rows: List<com.intrada.shared.ReflectionTempoView>) {
    val marks =
        mutableStateMapOf<String, Int>().apply {
            putAll(seed.marks.associate { it.playId to it.score.toInt() })
        }
    var note by mutableStateOf(seed.note)
    var draftedNote by mutableStateOf(seed.note)
    /** Only a tempo the musician moved is sent (#1420); one set before a resume stays set. */
    val tempos =
        mutableStateMapOf<String, Pair<Int, Boolean>>().apply {
            putAll(
                rows.associate { row ->
                    val tempo =
                        row.tempo
                            .toInt()
                            .coerceAtLeast(row.band.min.toInt())
                            .coerceAtMost(row.band.max.toInt())
                    row.playId to (tempo to row.setByHand)
                }
            )
        }
    var felt by mutableStateOf(seed.felt)
    var obstacles by mutableStateOf(seed.gotInTheWay)
    var intentionMet by mutableStateOf(seed.intentionMet)
    var detailOpen by mutableStateOf(seed.felt != null || seed.gotInTheWay.isNotEmpty())
    private val notePoints = seed.notePoints
    private val ways = seed.ways

    fun answers(plays: List<PlayView>, rows: List<com.intrada.shared.ReflectionTempoView>) =
        ReflectionAnswers(
            marks =
                plays.mapNotNull { play ->
                    marks[play.id]?.let { DraftMark(play.id, it.toUByte()) }
                },
            note = note,
            tempos =
                rows.mapNotNull { row ->
                    tempos[row.playId]
                        ?.takeIf { it.second }
                        ?.let { DraftTempo(row.playId, it.first.toUShort(), row.click) }
                },
            felt = felt,
            gotInTheWay = obstacles,
            notePoints = notePoints,
            intentionMet = intentionMet,
            ways = ways,
        )
}

class ReflectionModel(
    val itemTitle: String,
    val reflection: ReflectionView,
    val plays: List<PlayView>,
    val finish: FinishSheetView?,
    val aim: String?,
    val limits: LimitsView,
) {
    val scoreRange: IntRange
        get() = limits.scoreMin.toInt()..limits.scoreMax.toInt()

    val tempoStep: Int
        get() = limits.clickTempoStep.toInt()
}

@Composable
internal fun ReflectionRoute(
    model: PlayerModel,
    reflection: ReflectionView,
    send: (Event) -> Boolean,
    modifier: Modifier = Modifier,
) {
    val active = model.active
    val entry = active.entries.getOrNull(active.currentPosition.toInt())
    var refusal by remember(entry?.id) { mutableStateOf<String?>(null) }
    // Cleared from the core, or it would show again on the player or summary once the sheet closes
    // (#2009).
    val refuse = {
        val now = model.alertsNow()
        refusal = if (now.halted) Store.HALTED_MESSAGE else now.error ?: "Couldn't save. Try again."
        send(Event.ClearError)
    }
    ReflectionSheet(
        ReflectionModel(
            itemTitle = active.currentItemTitle,
            reflection = reflection,
            plays = entry?.plays.orEmpty(),
            finish = active.record.finish,
            aim = entry?.record?.focus?.label ?: active.currentItemIntention,
            limits = model.limits,
        ),
        remember(entry?.id) { ReflectionState(reflection.answers, reflection.tempos) },
        ReflectionActions(
            onDraft = { send(Event.Session(SessionEvent.UpdateReflectionDraft(it))) },
            onSave = {
                if (!send(Event.Session(SessionEvent.SubmitReflection(SessionClock.now(), it)))) {
                    refuse()
                }
            },
            onSkip = {
                val now = SessionClock.now()
                if (!send(Event.Session(SessionEvent.NextItem(now, now, reflection.reading)))) {
                    refuse()
                }
            },
        ),
        modifier,
        refusal = refusal,
    )
}

class ReflectionActions(
    val onDraft: (ReflectionAnswers) -> Unit,
    val onSave: (ReflectionAnswers) -> Unit,
    val onSkip: () -> Unit,
)

@Composable
fun ReflectionSheet(
    model: ReflectionModel,
    state: ReflectionState,
    actions: ReflectionActions,
    modifier: Modifier = Modifier,
    refusal: String? = null,
) {
    val rows = model.reflection.tempos
    val draft = {
        val answers = state.answers(model.plays, rows)
        state.draftedNote = answers.note
        actions.onDraft(answers)
    }
    val draftNoteIfChanged = { if (state.note != state.draftedNote) draft() }
    LaunchedEffect(state.note) {
        delay(NOTE_PAUSE_MS)
        draftNoteIfChanged()
    }
    LifecycleEventEffect(Lifecycle.Event.ON_STOP) { draftNoteIfChanged() }
    // The item is over; back would only lose the answers, so it saves nothing and stays.
    BackHandler {}
    Column(
        modifier
            .fillMaxSize()
            .background(IntradaGradient.paper)
            .windowInsetsPadding(WindowInsets.safeDrawing.union(WindowInsets.ime))
            .verticalScroll(rememberScrollState())
            .padding(horizontal = IntradaSpacing.section)
            .padding(bottom = IntradaSpacing.section)
    ) {
        Heading(model)
        AimSection(model, state, draft)
        MarkSection(model, state, draft)
        FieldLabel("Reflection · optional", Modifier.padding(top = IntradaSpacing.card))
        FormField(
            "",
            state.note,
            { state.note = it },
            "reflection.note",
            Modifier.padding(top = IntradaSpacing.controlGap).cardSurface(IntradaRadius.control),
            placeholder = "What went well? What to fix next time?",
            singleLine = false,
        )
        model.finish?.let { DetailSection(it, state, draft) }
        refusal?.let {
            FormErrorBanner(
                it,
                Modifier.padding(top = IntradaSpacing.card).semantics {
                    liveRegion = LiveRegionMode.Polite
                },
            )
        }
        InkButton(
            "Save & continue",
            "reflection.save",
            { actions.onSave(state.answers(model.plays, rows)) },
            Modifier.padding(top = IntradaSpacing.card),
        )
        Row(Modifier.fillMaxWidth().padding(top = IntradaSpacing.controlGap)) {
            TextAction("Skip rating", "reflection.skip", actions.onSkip, Modifier.weight(1f))
        }
    }
}

private const val NOTE_PAUSE_MS = 600L

@Composable
private fun Heading(model: ReflectionModel) {
    Column(
        Modifier.fillMaxWidth().padding(top = IntradaSpacing.card),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
    ) {
        FieldLabel(
            "Item complete · ${SessionClock.clockDisplay(model.reflection.elapsedSecs.toLong())}"
        )
        BasicText(
            "How did it go?",
            style = IntradaFont.title.copy(color = IntradaColor.ink, textAlign = TextAlign.Center),
        )
        BasicText(
            model.itemTitle,
            style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
private fun AimSection(model: ReflectionModel, state: ReflectionState, draft: () -> Unit) {
    val finish = model.finish
    val aim = model.aim
    val read = finish?.intentionMetRead
    if (finish == null || aim == null || !finish.hasAim) return
    Column(
        Modifier.padding(top = IntradaSpacing.section),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
    ) {
        FieldLabel(if (finish.asksIntention) "Aim met? · optional" else "Aim")
        BasicText(aim, style = IntradaFont.bodyMedium.copy(color = IntradaColor.ink))
        if (finish.asksIntention) {
            ChoicePills(
                IntentionMet.entries,
                chosen = { it == state.intentionMet },
                label = { it.answer },
                tag = { "reflection.aim.${it.name.lowercase()}" },
                onTap = {
                    state.intentionMet = if (state.intentionMet == it) null else it
                    draft()
                },
            )
        } else if (read != null) {
            BasicText(
                read.readLabel,
                style = IntradaFont.secondary.copy(color = IntradaColor.success),
            )
        }
    }
}

private val FinishSheetView.hasAim: Boolean
    get() = asksIntention || intentionMetRead != null

private val IntentionMet.answer: String
    get() =
        when (this) {
            IntentionMet.YES -> "Yes"
            IntentionMet.PARTLY -> "Partly"
            IntentionMet.NOTYET -> "Not yet"
        }

private val IntentionMet.readLabel: String
    get() =
        when (this) {
            IntentionMet.YES -> "Met, from your plays"
            IntentionMet.PARTLY -> "Partly met, from your plays"
            IntentionMet.NOTYET -> "Not met yet, from your plays"
        }

@Composable
private fun MarkSection(model: ReflectionModel, state: ReflectionState, draft: () -> Unit) {
    val plays = model.plays
    val only = plays.singleOrNull()
    Column(
        Modifier.padding(top = IntradaSpacing.section),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
    ) {
        if (plays.size > 1) {
            FieldLabel("What you played")
            plays.forEachIndexed { index, play ->
                if (index > 0) HairlineDivider()
                Column(
                    Modifier.padding(vertical = IntradaSpacing.cardCompact),
                    verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
                ) {
                    val title = model.rowTitle(play)
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        BasicText(
                            title,
                            Modifier.weight(1f),
                            style = IntradaFont.bodyMedium.copy(color = IntradaColor.ink),
                        )
                        BasicText(
                            play.meta,
                            style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
                        )
                    }
                    if (play.isMarkable) PlayMarks(model, state, play, title, draft)
                }
            }
        } else if (only != null) {
            FieldLabel("Mark")
            PlayMarks(model, state, only, model.itemTitle, draft, tempoHeading = "Tempo reached")
        }
    }
}

@Composable
private fun PlayMarks(
    model: ReflectionModel,
    state: ReflectionState,
    play: PlayView,
    title: String,
    draft: () -> Unit,
    tempoHeading: String? = null,
) {
    val row = model.reflection.tempos.firstOrNull { it.playId == play.id }
    val bpm = state.tempos[play.id]?.first
    Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap)) {
        ScoreSelector(
            state.marks[play.id] ?: 0,
            model.scoreRange,
            "Mark for $title",
            "reflection.mark",
            onSelect = { next ->
                if (next == null) state.marks.remove(play.id)
                else state.marks[play.id] = next.toInt()
                draft()
            },
        )
        if (row != null && bpm != null) {
            if (tempoHeading != null) {
                FieldLabel(tempoHeading, Modifier.padding(top = IntradaSpacing.controlGap))
            }
            StepperRow(
                "$bpm bpm",
                "Tempo for $title",
                "reflection.tempo",
                onStep = { direction ->
                    state.tempos[play.id] =
                        (bpm + direction * model.tempoStep).coerceIn(
                            row.band.min.toInt(),
                            row.band.max.toInt(),
                        ) to true
                    draft()
                },
                canDecrease = bpm > row.band.min.toInt(),
                canIncrease = bpm < row.band.max.toInt(),
            )
        }
    }
}

private fun ReflectionModel.rowTitle(play: PlayView): String =
    finish?.rows?.firstOrNull { it.playId == play.id }?.label ?: play.label ?: "No variation"

private val PlayView.meta: String
    get() =
        listOfNotNull(
                SessionClock.clockDisplay(seconds.toLong()),
                repTarget?.let { "${repCount ?: 0u} of $it" },
            )
            .joinToString(" · ")

@Composable
private fun DetailSection(finish: FinishSheetView, state: ReflectionState, draft: () -> Unit) {
    Column(
        Modifier.padding(top = IntradaSpacing.card),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
    ) {
        ToggleRow(
            "Add detail",
            on = state.detailOpen,
            tag = "reflection.addDetail",
            onChange = { state.detailOpen = it },
        )
        if (!state.detailOpen) return@Column
        FieldLabel("How it felt")
        ChoicePills(
            finish.feltChoices.map { it.felt },
            chosen = { it == state.felt },
            label = { felt -> finish.feltChoices.first { it.felt == felt }.label },
            tag = { "reflection.felt.${it.name.lowercase()}" },
            onTap = { felt: Felt ->
                state.felt = if (state.felt == felt) null else felt
                draft()
            },
        )
        FieldLabel("What got in the way", Modifier.padding(top = IntradaSpacing.controlGap))
        ChoicePills(
            finish.obstacleChoices.map { it.obstacle },
            chosen = { it in state.obstacles },
            label = { obstacle -> finish.obstacleChoices.first { it.obstacle == obstacle }.label },
            tag = { "reflection.obstacle.${it.name.lowercase()}" },
            onTap = { obstacle: Obstacle ->
                state.obstacles =
                    if (obstacle in state.obstacles) state.obstacles - obstacle
                    else state.obstacles + obstacle
                draft()
            },
        )
    }
}
