package com.intrada.android

import android.content.Context
import android.content.Intent
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.test.ComposeTimeoutException
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.SemanticsNodeInteraction
import androidx.compose.ui.test.hasAnyDescendant
import androidx.compose.ui.test.hasContentDescription
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.isRoot
import androidx.compose.ui.test.junit4.ComposeTestRule
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.printToString
import androidx.core.content.edit
import androidx.lifecycle.Lifecycle
import androidx.test.core.app.ActivityScenario
import androidx.test.platform.app.InstrumentationRegistry
import com.intrada.android.core.Settings
import org.junit.Assert.fail

/** The whole app on the emulator, launched the way `just android-run` launches it. */
class App(private val compose: ComposeTestRule) {
    private var scenario: ActivityScenario<MainActivity>? = null

    fun launch(seed: Boolean) {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        val intent =
            Intent(context, MainActivity::class.java).putExtra(MainActivity.EXTRA_SEED, seed)
        scenario = ActivityScenario.launch(intent)
    }

    /** An empty store and no saved settings, so the welcome is up whatever an earlier run left. */
    fun launchFresh() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        context.deleteDatabase(StoreHolder.DATABASE)
        for (name in listOf(Settings.PREFERENCES, Settings.PRACTICE_PREFERENCES)) {
            context.getSharedPreferences(name, Context.MODE_PRIVATE).edit(commit = true) { clear() }
        }
        launch(seed = false)
    }

    fun relaunch() {
        close()
        launch(seed = false)
    }

    /** The musician leaves for the home screen: the activity stops but is not yet gone. */
    fun background() {
        scenario?.moveToState(Lifecycle.State.CREATED)
    }

    fun close() {
        scenario?.close()
        scenario = null
    }

    fun await(matcher: SemanticsMatcher, timeoutMillis: Long = TIMEOUT): SemanticsNodeInteraction {
        try {
            compose.waitUntil(timeoutMillis) {
                compose.onAllNodes(matcher).fetchSemanticsNodes().isNotEmpty()
            }
        } catch (timeout: ComposeTimeoutException) {
            // The screen as it stood, so a red run on CI says where the app got to.
            val screen =
                compose.onAllNodes(isRoot()).fetchSemanticsNodes().size.let { roots ->
                    (0 until roots).joinToString("\n") {
                        compose.onAllNodes(isRoot())[it].printToString()
                    }
                }
            throw AssertionError(
                "no ${matcher.description} within ${timeoutMillis}ms on:\n$screen",
                timeout,
            )
        }
        return compose.onAllNodes(matcher)[0]
    }

    fun control(tag: String, timeoutMillis: Long = TIMEOUT) = await(hasTestTag(tag), timeoutMillis)

    fun tap(tag: String, timeoutMillis: Long = TIMEOUT) {
        control(tag, timeoutMillis).scrolledTo().performClick()
    }

    fun row(tag: String, containing: String) =
        await(hasTestTag(tag) and (mentions(containing) or hasAnyDescendant(mentions(containing))))

    fun tapRow(tag: String, containing: String) {
        row(tag, containing).scrolledTo().performClick()
    }

    fun text(text: String, timeoutMillis: Long = TIMEOUT) =
        await(hasText(text, substring = true), timeoutMillis)

    fun tapTab(tag: String) = control(tag).performClick()

    fun assertNeverShows(matcher: SemanticsMatcher, withinMillis: Long, what: String) {
        val shown =
            try {
                compose.waitUntil(withinMillis) {
                    compose.onAllNodes(matcher).fetchSemanticsNodes().isNotEmpty()
                }
                true
            } catch (_: ComposeTimeoutException) {
                false
            }
        if (shown) fail(what)
    }

    fun assertGone(matcher: SemanticsMatcher, what: String) {
        try {
            compose.waitUntil(TIMEOUT) {
                compose.onAllNodes(matcher).fetchSemanticsNodes().isEmpty()
            }
        } catch (_: ComposeTimeoutException) {
            fail(what)
        }
    }

    companion object {
        /** Scrolls only inside a scrolling parent: a toolbar action has none to scroll. */
        fun SemanticsNodeInteraction.scrolledTo(): SemanticsNodeInteraction {
            var parent = fetchSemanticsNode().parent
            while (parent != null) {
                if (SemanticsActions.ScrollBy in parent.config) return performScrollTo()
                parent = parent.parent
            }
            return this
        }

        // Generous: the CI emulator runs without hardware acceleration for graphics.
        const val TIMEOUT = 15_000L

        fun mentions(text: String) =
            hasText(text, substring = true) or hasContentDescription(text, substring = true)
    }
}
