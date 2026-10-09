package com.intrada.android

import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.ui.FirstPieceStep
import com.intrada.android.ui.ProfileEditState
import com.intrada.android.ui.ProfileStep
import com.intrada.android.ui.WelcomeStep
import com.intrada.shared.HighlighterColour
import kotlinx.coroutines.test.runTest
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = RobolectricDeviceQualifiers.Pixel7)
class FirstRunSnapshotTest {
    @Test
    fun welcome() {
        captureRoboImage("src/test/snapshots/first-run-welcome.png") {
            WelcomeStep(onSetUpProfile = {}, onSkip = {})
        }
    }

    @Test
    fun profile() = runTest {
        val profile = checkNotNull(openedStore(InMemoryItemStore()).viewModel.value).profile
        val form = ProfileEditState("Clara", "Cello", null, HighlighterColour.MINT)
        captureRoboImage("src/test/snapshots/first-run-profile.png") {
            ProfileStep(form, profile, onSave = {}, onSkip = {})
        }
    }

    @Test
    fun firstPiece() {
        captureRoboImage("src/test/snapshots/first-run-first-piece.png") {
            FirstPieceStep(onAdd = {}, onSkip = {})
        }
    }
}
