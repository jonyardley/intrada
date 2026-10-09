package com.intrada.android.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.unit.dp
import com.intrada.android.R
import com.intrada.shared.SuggestedPlan

class UpNextActions(
    val onStart: () -> Unit = {},
    val onChange: () -> Unit = {},
    val onBuildOwn: () -> Unit = {},
    val onPriorities: () -> Unit = {},
)

// Dismissal is not a one-way door while the core still has a plan to offer (#1618).
fun showsSuggestionRestore(plan: SuggestedPlan?, idle: Boolean, dismissed: Boolean): Boolean =
    dismissed && plan != null && idle

// Text, never a filled button: two filled buttons a thumb apart that both start a session is the
// ambiguity T15 rejected for the Up next card (T20, docs/design-principles.md).
@Composable
internal fun PrioritiesButton(onTap: () -> Unit) {
    IconLink(
        "Practise your priorities",
        R.drawable.ic_star,
        "Builds a session from everything you have starred",
        "practice.priorities",
        onTap,
    )
}

@Composable
internal fun ShowSuggestionButton(onTap: () -> Unit) {
    IconLink(
        "Show suggestion",
        R.drawable.ic_restore,
        "Brings back the suggested session",
        "practice.showSuggestion",
        onTap,
        muted = true,
    )
}

@Composable
private fun IconLink(
    text: String,
    icon: Int,
    hint: String,
    tag: String,
    onTap: () -> Unit,
    muted: Boolean = false,
) {
    val tint = if (muted) IntradaColor.inkSecondary else IntradaColor.accent
    val style = (if (muted) IntradaFont.secondary else IntradaFont.button).copy(color = tint)
    Row(
        Modifier.fillMaxWidth()
            .heightIn(min = 48.dp)
            .clickable(role = Role.Button, onClickLabel = hint, onClick = onTap)
            .testTag(tag),
        horizontalArrangement =
            Arrangement.spacedBy(IntradaSpacing.controlGap, Alignment.CenterHorizontally),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Image(
            painterResource(icon),
            contentDescription = null,
            modifier = Modifier.size(IntradaIconSize.inline.points),
            colorFilter = ColorFilter.tint(tint),
        )
        BasicText(text, style = style)
    }
}
