package com.intrada.android

import androidx.compose.ui.test.ComposeTimeoutException
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.hasContentDescription
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.isRoot
import androidx.compose.ui.test.junit4.ComposeTestRule
import androidx.compose.ui.test.printToString
import com.intrada.android.ui.AppTab
import org.junit.Assert.fail

/** The seeded Up next is on screen, so its "Build my own instead" opens a blank builder. */
fun App.openEmptyBuilder() {
    tapTab(AppTab.PRACTICE.tag)
    tap("practice.buildOwn")
}

fun App.startOneItemSession() {
    openEmptyBuilder()
    tap("builder.addItems")
    tapRow("addToSession.row", "Major Scales")
    tap("addToSession.done")
    tap("builder.start")
}

/** A builder row speaks its title first, then its kind or "Related". */
fun builderRow(title: String, meta: String) =
    hasTestTag("builder.row") and
        hasContentDescription("$title, ", substring = true) and
        hasContentDescription(meta, substring = true)

fun ComposeTestRule.awaitCount(matcher: SemanticsMatcher, count: Int, what: String) {
    try {
        waitUntil(App.TIMEOUT) { onAllNodes(matcher).fetchSemanticsNodes().size == count }
    } catch (_: ComposeTimeoutException) {
        fail("$what: ${onAllNodes(matcher).fetchSemanticsNodes().size}, not $count")
    }
}

fun ComposeTestRule.settles(timeoutMillis: Long, condition: () -> Boolean): Boolean =
    try {
        waitUntil(timeoutMillis, condition)
        true
    } catch (_: ComposeTimeoutException) {
        false
    }

fun ComposeTestRule.awaitThat(what: String, condition: () -> Boolean) {
    if (settles(App.TIMEOUT, condition)) return
    val roots = onAllNodes(isRoot())
    val screen =
        (0 until roots.fetchSemanticsNodes().size).joinToString("\n") { roots[it].printToString() }
    throw AssertionError("not $what within ${App.TIMEOUT}ms on:\n$screen")
}
