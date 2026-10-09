package com.intrada.android.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.testTag
import androidx.compose.ui.unit.dp
import com.intrada.android.R
import com.intrada.android.ui.components.dropShadow
import com.intrada.android.ui.components.label
import com.intrada.shared.HighlighterColour
import com.intrada.shared.ItemKind
import com.intrada.shared.SuggestedItem
import com.intrada.shared.SuggestedPlan
import com.intrada.shared.SuggestedSession

// Every reason string is the core's: the shell renders, never composes (design-principles T15).
@Composable
internal fun UpNextCard(plan: SuggestedPlan, colour: HighlighterColour, actions: UpNextActions) {
    val lead = plan.blocks.firstOrNull() ?: return
    val heroShape = RoundedCornerShape(IntradaRadius.hero)
    Column(
        Modifier.fillMaxWidth()
            .dropShadow(IntradaShadow.hero, heroShape)
            .clip(heroShape)
            .background(IntradaGradient.practiceHero(colour))
            .padding(IntradaSpacing.section)
            .testTag("practice.upNext"),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
    ) {
        Headline(plan, lead, colour)
        WashedList(filled = true, lead.items.map { item -> { ItemRow(item) } })
        if (plan.blocks.size > 1) LaterList(plan.blocks.drop(1))
        StartButton(plan, colour, actions.onStart)
        SecondaryActions(actions)
    }
}

private val SuggestedPlan.heading: String
    get() = if (lengthMins == null) "Up next" else "Today's plan"

private val SuggestedPlan.countLabel: String
    get() = "${itemsLabel(itemCount.toInt())} · $estimatedMinutes min"

private fun itemsLabel(count: Int) = "$count item${if (count == 1) "" else "s"}"

// TalkBack reads the house separator as "middle dot"; commas are the pause the sentence wants.
private fun spoken(text: String) = text.replace(" · ", ", ")

// Filled to a length, the totals are the plan's, so they come before the lead piece.
private fun headlineLabel(plan: SuggestedPlan, lead: SuggestedSession): String {
    val count = "${itemsLabel(plan.itemCount.toInt())}, ${plan.estimatedMinutes} minutes"
    val filled = plan.lengthMins != null
    return listOfNotNull(
            plan.heading,
            count.takeIf { filled },
            lead.pieceTitle,
            lead.pieceSubtitle,
            spoken(lead.reason),
            count.takeUnless { filled },
        )
        .joinToString(", ")
}

private val onHeroSecondary = IntradaColor.onAccent.copy(alpha = IntradaOpacity.secondary)
private val wash = IntradaColor.paperTop.copy(alpha = IntradaOpacity.wash)

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun Headline(plan: SuggestedPlan, lead: SuggestedSession, colour: HighlighterColour) {
    Column(
        Modifier.fillMaxWidth().clearAndSetSemantics {
            contentDescription = headlineLabel(plan, lead)
        },
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
    ) {
        FlowRow(
            Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalArrangement = Arrangement.spacedBy(2.dp),
        ) {
            BasicText(plan.heading, style = IntradaFont.label.copy(color = onHeroSecondary))
            BasicText(plan.countLabel, style = IntradaFont.secondary.copy(color = onHeroSecondary))
        }
        BasicText(
            lead.pieceTitle,
            style = IntradaFont.title.copy(color = IntradaColor.paperTop),
            maxLines = 3,
        )
        Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            if (lead.priority) {
                Image(
                    painterResource(R.drawable.ic_star),
                    contentDescription = null,
                    modifier = Modifier.padding(top = 4.dp).size(IntradaIconSize.caption.points),
                    colorFilter = ColorFilter.tint(IntradaColor.marker(colour)),
                )
            }
            BasicText(
                lead.reason,
                style =
                    IntradaFont.secondary.copy(
                        color = IntradaColor.onAccent.copy(alpha = IntradaOpacity.strong)
                    ),
            )
        }
    }
}

@Composable
private fun WashedList(filled: Boolean, rows: List<@Composable () -> Unit>) {
    val shape = RoundedCornerShape(IntradaRadius.card)
    Column(
        Modifier.fillMaxWidth()
            .clip(shape)
            .background(if (filled) wash else Color.Transparent)
            .border(1.dp, wash, shape)
            .padding(horizontal = IntradaSpacing.cardCompact)
    ) {
        rows.forEachIndexed { index, row ->
            if (index > 0) Box(Modifier.fillMaxWidth().heightIn(min = 1.dp).background(wash))
            row()
        }
    }
}

private val ItemKind.onHeroDot: Color
    get() =
        when (this) {
            ItemKind.PIECE -> IntradaColor.onHeroPiece
            ItemKind.EXERCISE -> IntradaColor.onHeroExercise
        }

@Composable
private fun ItemRow(item: SuggestedItem) {
    val spokenRow =
        listOfNotNull(item.itemTitle, item.itemType.label, spoken(item.reason), item.weakestSection)
            .joinToString(", ")
    Row(
        Modifier.fillMaxWidth()
            .padding(vertical = IntradaSpacing.cardCompact)
            .clearAndSetSemantics { contentDescription = spokenRow },
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
    ) {
        Box(
            Modifier.padding(top = 7.dp)
                .size(7.dp)
                .clip(CircleShape)
                .background(item.itemType.onHeroDot)
        )
        Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
            BasicText(
                item.itemTitle,
                style = IntradaFont.bodyMedium.copy(color = IntradaColor.paperTop),
            )
            BasicText(item.reason, style = IntradaFont.secondary.copy(color = onHeroSecondary))
            item.weakestSection?.let {
                BasicText(it, style = IntradaFont.secondary.copy(color = onHeroSecondary))
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun LaterList(blocks: List<SuggestedSession>) {
    Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap)) {
        BasicText("Then", style = IntradaFont.label.copy(color = onHeroSecondary))
        WashedList(
            filled = false,
            blocks.map { block ->
                {
                    val totals = "${itemsLabel(block.items.size)} · ${block.estimatedMinutes} min"
                    FlowRow(
                        Modifier.fillMaxWidth()
                            .padding(vertical = IntradaSpacing.cardCompact)
                            .clearAndSetSemantics {
                                contentDescription = "${block.pieceTitle}, ${spoken(totals)}"
                            },
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalArrangement = Arrangement.spacedBy(2.dp),
                    ) {
                        BasicText(
                            block.pieceTitle,
                            style = IntradaFont.bodyMedium.copy(color = IntradaColor.paperTop),
                        )
                        BasicText(
                            totals,
                            style = IntradaFont.secondary.copy(color = onHeroSecondary),
                        )
                    }
                }
            },
        )
    }
}

@Composable
private fun StartButton(plan: SuggestedPlan, colour: HighlighterColour, onStart: () -> Unit) {
    val shape = RoundedCornerShape(IntradaRadius.control)
    val spokenValue =
        "${itemsLabel(plan.itemCount.toInt())}, about ${plan.estimatedMinutes} minutes"
    Row(
        Modifier.fillMaxWidth()
            .heightIn(min = 48.dp)
            .clip(shape)
            .background(IntradaColor.marker(colour))
            .clickable(onClick = onStart)
            .clearAndSetSemantics {
                testTag = "practice.start"
                contentDescription = "Start practising, $spokenValue"
                role = Role.Button
                onClick {
                    onStart()
                    true
                }
            }
            .padding(vertical = IntradaSpacing.card),
        horizontalArrangement =
            Arrangement.spacedBy(IntradaSpacing.controlGap, Alignment.CenterHorizontally),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Image(
            painterResource(R.drawable.ic_play),
            contentDescription = null,
            modifier = Modifier.size(IntradaIconSize.inline.points),
            colorFilter = ColorFilter.tint(IntradaColor.onMarker),
        )
        BasicText(
            "Start · ${plan.estimatedMinutes} min",
            style = IntradaFont.button.copy(color = IntradaColor.onMarker),
        )
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun SecondaryActions(actions: UpNextActions) {
    FlowRow(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
        HeroLink(
            "Change it first",
            "practice.changePlan",
            "open the session builder with this plan in it",
            actions.onChange,
        )
        HeroLink(
            "Build my own instead",
            "practice.buildOwn",
            "open the session builder",
            actions.onBuildOwn,
        )
    }
}

@Composable
private fun HeroLink(text: String, tag: String, hint: String, onTap: () -> Unit) {
    Box(
        Modifier.heightIn(min = 48.dp)
            .clickable(role = Role.Button, onClickLabel = hint, onClick = onTap)
            .testTag(tag),
        contentAlignment = Alignment.CenterStart,
    ) {
        BasicText(text, style = IntradaFont.secondary.copy(color = onHeroSecondary))
    }
}
