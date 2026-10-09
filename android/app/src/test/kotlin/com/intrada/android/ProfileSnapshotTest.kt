package com.intrada.android

import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.core.Store
import com.intrada.android.ui.InstrumentIconPicker
import com.intrada.android.ui.IntradaColor
import com.intrada.android.ui.ProfileActions
import com.intrada.android.ui.ProfileEditSheet
import com.intrada.android.ui.ProfileEditState
import com.intrada.android.ui.ProfileRefusal
import com.intrada.android.ui.ProfileScreen
import com.intrada.android.ui.saveProfile
import com.intrada.shared.HighlighterColour
import com.intrada.shared.InstrumentIcon
import com.intrada.shared.Profile
import com.intrada.shared.ProfileField
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = RobolectricDeviceQualifiers.Pixel7)
class ProfileSnapshotTest {
    private val actions = ProfileActions(onEdit = {}, onDefaults = {}, onFeedback = {})

    @Test
    @Config(qualifiers = "w411dp-h1300dp-xxhdpi")
    fun profile() = runTest {
        val store = profiledStore()
        val view = checkNotNull(store.viewModel.value)
        captureRoboImage("src/test/snapshots/profile.png") {
            ProfileScreen(view.profile, view.practiceDefaults, view.limits, actions)
        }
    }

    @Test
    fun profileEmpty() = runTest {
        val view = checkNotNull(openedStore().viewModel.value)
        captureRoboImage("src/test/snapshots/profile-empty.png") {
            ProfileScreen(view.profile, view.practiceDefaults, view.limits, actions)
        }
    }

    @Test
    fun editSheet() = runTest {
        val profile = checkNotNull(profiledStore().viewModel.value).profile
        captureRoboImage("src/test/snapshots/profile-edit.png") {
            ProfileEditSheet(ProfileEditState.of(profile), profile, onCancel = {}, onSave = {})
        }
    }

    @Test
    fun editSheetRefused() = runTest {
        val profile = checkNotNull(profiledStore().viewModel.value).profile
        val form =
            ProfileEditState.of(profile).apply {
                instrument = "c".repeat(101)
                refusal =
                    ProfileRefusal(
                        "Instrument must be 100 characters or fewer",
                        ProfileField.INSTRUMENT,
                    )
            }
        captureRoboImage("src/test/snapshots/profile-edit-refused.png") {
            ProfileEditSheet(form, profile, onCancel = {}, onSave = {})
        }
    }

    @Test
    fun iconPicker() {
        captureRoboImage("src/test/snapshots/profile-icon-picker.png") {
            InstrumentIconPicker(
                InstrumentIcon.CELLO,
                InstrumentIcon.HARP,
                IntradaColor.marker(HighlighterColour.MINT),
                onChoose = {},
                onDone = {},
            )
        }
    }

    private suspend fun TestScope.profiledStore(): Store {
        val store = openedStore()
        store.saveProfile(Profile("Clara", "Cello", null, HighlighterColour.MINT))?.let {
            error("the fixture profile was refused: ${it.message}")
        }
        return store
    }
}
