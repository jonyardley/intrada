package com.intrada.android

import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import com.intrada.android.ui.AppTab

/**
 * An empty store with one piece typed in through the welcome, so the practice it plays is kept: a
 * seeded launch keeps no settings, so nothing it plays can be recovered.
 */
fun App.launchWithOnePiece(title: String) {
    launchFresh()
    tap("firstRun.setUpProfile")
    tap("firstRun.skipProfile")
    tap("firstRun.typePiece")
    control("itemForm.title").performTextInput(title)
    tap("itemForm.confirm")
    tapTab(AppTab.PRACTICE.tag)
}

/** From the open builder: the one item added and the session started. */
fun App.startWith(title: String) {
    tap("builder.addItems")
    tapRow("addToSession.row", title)
    tap("addToSession.done")
    tap("builder.start")
    control("player.title")
}

fun App.endSessionEarly() {
    tap("player.options")
    tap("player.options.end")
}

fun App.discardSummary() {
    tap("summary.discard")
    tap("summary.discardConfirm.confirm")
    assertGone(hasTestTag("summary.discard"), "the discarded session leaves the summary")
}

// The screen after a relaunch can take well past the usual wait on a loaded machine (#1923).
const val CONTENDED = 30_000L

fun App.relaunchToPractice() {
    relaunch()
    control(AppTab.PRACTICE.tag, CONTENDED).performClick()
}

fun App.resumeAfterRelaunch() {
    relaunchToPractice()
    tap("practice.resume", CONTENDED)
}
