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
import com.intrada.android.core.Store
import com.intrada.android.ui.BannerAction
import com.intrada.android.ui.BannerTone
import com.intrada.android.ui.ConsistencyBars
import com.intrada.android.ui.FormField
import com.intrada.android.ui.GlobalBanner
import com.intrada.android.ui.HeaderAction
import com.intrada.android.ui.IntradaColor
import com.intrada.android.ui.IntradaFont
import com.intrada.android.ui.IntradaGlyph
import com.intrada.android.ui.IntradaSpacing
import com.intrada.android.ui.MasteryDeltaToast
import com.intrada.android.ui.MasteryDial
import com.intrada.android.ui.ScoreRing
import com.intrada.android.ui.ScoreSelector
import com.intrada.android.ui.ScreenScaffold
import com.intrada.android.ui.SectionHeader
import com.intrada.android.ui.SegmentedPills
import com.intrada.android.ui.SessionCard
import com.intrada.android.ui.TextAction
import com.intrada.android.ui.WeekStrip
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
    fun segmentedPills() {
        captureRoboImage("src/test/snapshots/segmented-pills.png") {
            Paper {
                Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.section)) {
                    SegmentedPills(
                        listOf("Piece", "Exercise"),
                        "Exercise",
                        {},
                        label = { it },
                        tag = { it },
                    )
                    SegmentedPills(
                        listOf("2 + 2 + 3", "2 + 3 + 2", "3 + 2 + 2"),
                        "2 + 3 + 2",
                        {},
                        label = { it },
                        tag = { it },
                        wraps = true,
                    )
                }
            }
        }
    }

    @Test
    fun scoreRings() {
        captureRoboImage("src/test/snapshots/score-ring.png") {
            Paper {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    listOf(1, 4, 7, 10).forEach { ScoreRing(it, topMark = 10) }
                    ScoreRing(7, topMark = 10, size = 34.dp)
                }
            }
        }
    }

    @Test
    fun globalBanners() {
        captureRoboImage("src/test/snapshots/global-banner.png") {
            Column(Modifier.width(360.dp).background(IntradaColor.paperTop)) {
                GlobalBanner(
                    "Session deleted",
                    tag = "banner.undoDelete",
                    tone = BannerTone.NOTICE,
                    action = BannerAction("Undo") {},
                    onDismiss = {},
                )
                GlobalBanner(Store.HALTED_MESSAGE, tag = "banner.halted")
                GlobalBanner("Couldn't delete that item.", tag = "banner.error", onDismiss = {})
            }
        }
    }

    @Test
    fun screenScaffold() {
        captureRoboImage("src/test/snapshots/screen-scaffold.png") {
            ScreenScaffold(
                "Clair de Lune",
                Modifier.width(360.dp).height(240.dp),
                actions = { TextAction("Edit", "piece.edit", {}) },
            ) {
                BasicText(
                    "Claude Debussy",
                    Modifier.fillMaxWidth().padding(IntradaSpacing.card),
                    style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
                )
            }
        }
    }

    @Test
    fun sectionHeaders() {
        captureRoboImage("src/test/snapshots/section-header.png") {
            Paper {
                Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.section)) {
                    SectionHeader("Recently practised")
                    SectionHeader("Keys", caption = "2 of 4 solid")
                    SectionHeader("Last five weeks", trailing = "best week · 95m")
                    SectionHeader(
                        "Chord chart",
                        action = HeaderAction("Edit", "Edit chord chart", "chart.header") {},
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

    @Test
    fun scoreSelectors() {
        captureRoboImage("src/test/snapshots/score-selector.png") {
            Paper {
                Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.section)) {
                    listOf(0, 4, 10).forEach { score ->
                        ScoreSelector(score, 1..10, "Mark for Clair de Lune", "mark", {})
                    }
                }
            }
        }
    }

    @Test
    fun masteryDeltaToasts() {
        captureRoboImage("src/test/snapshots/mastery-delta-toast.png") {
            Paper { MasteryDeltaToast("Mastery up", "Clair de Lune", was = 5, now = 7) }
        }
    }

    @Test
    fun sessionCards() {
        captureRoboImage("src/test/snapshots/session-card.png") {
            Paper {
                Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact)) {
                    SessionCard(PracticeFixtures.completed, onOpen = {})
                    SessionCard(PracticeFixtures.withVariations, onOpen = {})
                }
            }
        }
    }

    @Test
    fun weekStrip() {
        val week = PracticeFixtures.week
        captureRoboImage("src/test/snapshots/week-strip.png") {
            Paper { WeekStrip(week.days, week.days[week.openingDay.toInt()].date, {}) }
        }
    }

    @Test
    fun masteryDial() {
        captureRoboImage("src/test/snapshots/mastery-dial.png") {
            Paper { MasteryDial(MASTERY, topMark = TOP_MARK) }
        }
    }

    @Test
    fun consistencyBars() {
        captureRoboImage("src/test/snapshots/consistency-bars.png") {
            Paper { ConsistencyBars(ProgressFixtures.analytics.consistencyWeeks) }
        }
    }

    @Test
    fun formFields() {
        captureRoboImage("src/test/snapshots/form-field.png") { FormFields() }
    }

    @Composable
    private fun FormFields() {
        Paper {
            Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact)) {
                Column(Modifier.cardSurface()) {
                    FormField("Title", "Clair de Lune", {}, "form.title", placeholder = "Required")
                    HairlineDivider()
                    FormField("Tempo marking", "", {}, "form.marking", placeholder = "e.g. Allegro")
                    HairlineDivider()
                    FormField(
                        "Bars",
                        "",
                        {},
                        "form.bars",
                        placeholder = "1 to 16",
                        note = "Optional. Bar 12, or 12 to 14",
                    )
                }
                FormField(
                    "Title",
                    "",
                    {},
                    "form.faulted",
                    Modifier.cardSurface(),
                    placeholder = "Required",
                    faulted = true,
                )
            }
        }
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
                FormErrorBanner("Title must be between 1 and 500 characters")
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
        const val MASTERY = 3.4
        const val TOP_MARK = 10
    }
}
