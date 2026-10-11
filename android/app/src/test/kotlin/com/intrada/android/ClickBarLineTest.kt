package com.intrada.android

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.width
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.onPlaced
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.unit.dp
import com.intrada.android.ui.ClickBarLine
import com.intrada.android.ui.FirstThatFits
import com.intrada.shared.Metre
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class ClickBarLineTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun talkBackSaysWhatTappingTheBarDoes() {
        var taps = 0
        compose.setContent { ClickBarLine(Metre(4u, 4u), 0b1111u, null, { taps++ }) }

        val click =
            compose.onNodeWithTag("click.bar").fetchSemanticsNode().config[SemanticsActions.OnClick]
        assertEquals("Choose the time signature and which beats sound", click.label)
        compose.runOnIdle { click.action?.invoke() }
        assertEquals(1, taps)
    }

    @Test
    fun theFirstVersionThatFitsIsTheOneLaidOut() {
        val placed = mutableListOf<Int>()
        compose.setContent {
            Box(Modifier.width(250.dp)) {
                FirstThatFits(3) { index ->
                    Box(Modifier.width((300 - index * 100).dp).onPlaced { placed += index })
                }
            }
        }

        compose.runOnIdle { assertEquals(listOf(1), placed.distinct()) }
    }

    @Test
    fun theLastVersionIsLaidOutWhenNoneFits() {
        val placed = mutableListOf<Int>()
        compose.setContent {
            Box(Modifier.width(50.dp)) {
                FirstThatFits(2) { index ->
                    Box(Modifier.width((300 - index * 100).dp).onPlaced { placed += index })
                }
            }
        }

        compose.runOnIdle { assertEquals(listOf(1), placed.distinct()) }
    }
}
