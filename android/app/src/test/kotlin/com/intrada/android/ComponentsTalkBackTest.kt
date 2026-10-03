package com.intrada.android

import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertContentDescriptionEquals
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import com.intrada.android.ui.components.FormErrorBanner
import com.intrada.android.ui.components.InstrumentGlyph
import com.intrada.android.ui.components.TagChip
import com.intrada.shared.InstrumentIcon
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class ComponentsTalkBackTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun formErrorBannerSaysItIsAnError() {
        compose.setContent { FormErrorBanner("Title is required", Modifier.testTag(TAG)) }

        compose.onNodeWithTag(TAG).assertContentDescriptionEquals("Error: Title is required")
    }

    @Test
    fun removableTagChipIsAButtonThatRemoves() {
        var removed = 0
        compose.setContent { TagChip("Baroque", Modifier.testTag(TAG), onRemove = { removed++ }) }

        val chip = compose.onNodeWithTag(TAG)
        chip.assertContentDescriptionEquals("Baroque")
        val config = chip.fetchSemanticsNode().config
        assertEquals(Role.Button, config.getOrNull(SemanticsProperties.Role))
        val click = config[SemanticsActions.OnClick]
        assertEquals("remove", click.label)
        compose.runOnIdle { click.action?.invoke() }

        assertEquals(1, removed)
    }

    @Test
    fun instrumentGlyphNamesTheInstrumentFamily() {
        compose.setContent { InstrumentGlyph(InstrumentIcon.CELLO, Modifier.testTag(TAG)) }

        compose.onNodeWithTag(TAG).assertContentDescriptionEquals("Cello and double bass")
    }

    private companion object {
        const val TAG = "component"
    }
}
