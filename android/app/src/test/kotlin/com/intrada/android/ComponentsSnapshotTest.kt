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
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.dp
import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.ui.IntradaColor
import com.intrada.android.ui.IntradaSpacing
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.ScrimCapsule
import com.intrada.android.ui.components.TagChip
import com.intrada.android.ui.components.TagChipStyle
import com.intrada.android.ui.components.cardShadow
import com.intrada.android.ui.components.cardSurface
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
    fun tagChipsAtTheLargestFontScale() {
        captureRoboImage("src/test/snapshots/tag-chip-largest-font.png") {
            val density = LocalDensity.current
            CompositionLocalProvider(
                LocalDensity provides Density(density.density, fontScale = LARGEST_FONT_SCALE)
            ) {
                TagChips()
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
        const val LARGEST_FONT_SCALE = 2f
    }
}
