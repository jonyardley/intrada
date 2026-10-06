package com.intrada.android

import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.ui.IntradaColor
import com.intrada.android.ui.IntradaFont
import com.intrada.android.ui.IntradaGlyph
import com.intrada.android.ui.IntradaSpacing
import com.intrada.android.ui.components.FieldCard
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.FormErrorBanner
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.InstrumentGlyph
import com.intrada.android.ui.components.ProfileBadge
import com.intrada.android.ui.components.ScrimCapsule
import com.intrada.android.ui.components.TagChip
import com.intrada.android.ui.components.TagChipStyle
import com.intrada.android.ui.components.cardShadow
import com.intrada.android.ui.components.cardSurface
import com.intrada.shared.HighlighterColour
import com.intrada.shared.InstrumentIcon
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = RobolectricDeviceQualifiers.Pixel7)
class ComponentsSnapshotTest {
    @Test
    fun cardSurface() {
        captureRoboImage("src/test/snapshots/card-surface.png") {
            Paper { Box(Modifier.fillMaxWidth().height(80.dp).cardSurface()) }
        }
    }

    @Test
    fun cardShadow() {
        captureRoboImage("src/test/snapshots/card-shadow.png") {
            Paper { Box(Modifier.fillMaxWidth().height(80.dp).cardShadow().cardSurface()) }
        }
    }

    @Test
    fun hairlineDivider() {
        captureRoboImage("src/test/snapshots/hairline-divider.png") {
            Paper {
                Column(Modifier.cardSurface()) {
                    Box(Modifier.fillMaxWidth().height(40.dp))
                    HairlineDivider()
                    Row(Modifier.height(40.dp)) {
                        Box(Modifier.width(80.dp))
                        HairlineDivider(orientation = Orientation.Vertical)
                        Box(Modifier.width(80.dp))
                        HairlineDivider(
                            orientation = Orientation.Vertical,
                            colour = IntradaColor.divider,
                        )
                    }
                }
            }
        }
    }

    @Test
    fun scrimCapsule() {
        captureRoboImage("src/test/snapshots/scrim-capsule.png") {
            Box(
                Modifier.size(240.dp, 120.dp).background(Color.White).padding(IntradaSpacing.card)
            ) {
                ScrimCapsule("Done", onClick = {})
            }
        }
    }

    @Test
    fun tagChips() {
        captureRoboImage("src/test/snapshots/tag-chip.png") { TagChips() }
    }

    @Test
    fun instrumentGlyphs() {
        captureRoboImage("src/test/snapshots/instrument-glyph.png") {
            Paper {
                Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap)) {
                    InstrumentIcon.entries.chunked(GLYPHS_PER_ROW).forEach { row ->
                        Row(
                            horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap)
                        ) {
                            row.forEach { InstrumentGlyph(it, size = IntradaGlyph.bar) }
                        }
                    }
                    InstrumentGlyph(InstrumentIcon.CELLO)
                }
            }
        }
    }

    @Test
    fun profileBadges() {
        captureRoboImage("src/test/snapshots/profile-badge.png") {
            Paper {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    ProfileBadge(InstrumentIcon.CELLO, IntradaColor.marker, size = IntradaGlyph.bar)
                    ProfileBadge(InstrumentIcon.CELLO, IntradaColor.marker(HighlighterColour.MINT))
                    ProfileBadge(
                        InstrumentIcon.OTHER,
                        IntradaColor.marker(HighlighterColour.LAVENDER),
                        size = IntradaGlyph.hero,
                    )
                }
            }
        }
    }

    @Test
    fun formErrorBanner() {
        captureRoboImage("src/test/snapshots/form-error-banner.png") { FormErrorBanners() }
    }

    @Test
    fun fieldCards() {
        captureRoboImage("src/test/snapshots/field-card.png") { FieldCards() }
    }

    @Composable
    private fun FieldCards() {
        Paper {
            Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap)) {
                FieldLabel("Practice defaults")
                FieldCard("Beats in the bar") {
                    BasicText("4", style = IntradaFont.body.copy(color = IntradaColor.ink))
                }
                FieldCard("Notes") {
                    BasicText(
                        "Keep the left hand light through the second half",
                        style = IntradaFont.body.copy(color = IntradaColor.ink),
                    )
                }
            }
        }
    }

    @Composable
    private fun FormErrorBanners() {
        Paper {
            Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap)) {
                FormErrorBanner("Title is required")
                FormErrorBanner("Composer must be between 1 and 200 characters")
            }
        }
    }

    @Composable
    private fun TagChips() {
        Paper {
            Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap)) {
                Row(
                    Modifier.cardSurface().padding(IntradaSpacing.card),
                    horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    TagChip("Baroque")
                    TagChip("Sight-reading", onRemove = {})
                }
                Row(
                    horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    TagChip("Exam piece", style = TagChipStyle.Outlined)
                    TagChip("Left hand", style = TagChipStyle.Outlined, onRemove = {})
                }
            }
        }
    }

    @Composable
    private fun Paper(content: @Composable () -> Unit) {
        Box(
            Modifier.width(360.dp).background(IntradaColor.paperTop).padding(IntradaSpacing.section)
        ) {
            content()
        }
    }

    private companion object {
        const val GLYPHS_PER_ROW = 7
    }
}
