package com.intrada.android.ui

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.spring
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectHorizontalDragGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import com.intrada.android.R
import com.intrada.android.ui.components.scaled
import kotlin.math.roundToInt
import kotlinx.coroutines.launch

private val REVEAL_WIDTH = 88.dp
private val FULL_SWIPE = 180.dp

/**
 * A trailing swipe that reveals a delete button, the twin of the iPhone's. A long swipe deletes at
 * once. The gesture is an accelerator only (T4): the row offers the same action to TalkBack.
 */
@Composable
internal fun SwipeToDelete(
    label: String,
    onDelete: () -> Unit,
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit,
) {
    val density = LocalDensity.current
    val reveal = with(density) { REVEAL_WIDTH.toPx() }
    val full = with(density) { FULL_SWIPE.toPx() }
    val offset = remember { Animatable(0f) }
    // The gesture outlives recomposition, so it calls the latest delete (#2581).
    val currentDelete by rememberUpdatedState(onDelete)
    val scope = rememberCoroutineScope()
    val motion =
        spring<Float>(
            dampingRatio = IntradaMotion.snappy.dampingFraction,
            stiffness = IntradaMotion.snappy.stiffness,
        )
    val settle: (Float) -> Unit = { target -> scope.launch { offset.animateTo(target, motion) } }
    val perform = {
        settle(0f)
        currentDelete()
    }
    Box(modifier) {
        if (offset.value < 0f) {
            Box(Modifier.matchParentSize(), contentAlignment = Alignment.CenterEnd) {
                val width = with(density) { maxOf(reveal, -offset.value).toDp() }
                RevealedDelete(label, width, perform)
            }
        }
        Box(
            Modifier.offset { IntOffset(offset.value.roundToInt(), 0) }
                .pointerInput(Unit) {
                    detectHorizontalDragGestures(
                        onDragEnd = {
                            when {
                                offset.value <= -full -> perform()
                                offset.value < -reveal / 2 -> settle(-reveal)
                                else -> settle(0f)
                            }
                        },
                        onDragCancel = { settle(0f) },
                    ) { change, amount ->
                        change.consume()
                        scope.launch { offset.snapTo(minOf(0f, offset.value + amount)) }
                    }
                }
        ) {
            content()
        }
    }
}

@Composable
private fun RevealedDelete(label: String, width: Dp, onDelete: () -> Unit) {
    Box(
        Modifier.width(width)
            .fillMaxHeight()
            .clip(RoundedCornerShape(IntradaRadius.card))
            .background(IntradaColor.danger)
            .clickable(role = Role.Button, onClick = onDelete)
            .semantics { contentDescription = label },
        contentAlignment = Alignment.Center,
    ) {
        Image(
            painterResource(R.drawable.ic_trash),
            contentDescription = null,
            modifier = Modifier.size(IntradaIconSize.inline.scaled()),
            colorFilter = ColorFilter.tint(IntradaColor.onAccent),
        )
    }
}
