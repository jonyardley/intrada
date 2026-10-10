package com.intrada.android

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.ui.Modifier
import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Store
import com.intrada.android.ui.IntradaColor
import com.intrada.android.ui.IntradaSpacing
import com.intrada.android.ui.ItemFormExercises
import com.intrada.android.ui.ItemFormMode
import com.intrada.android.ui.ItemFormScreen
import com.intrada.android.ui.ItemFormState
import com.intrada.android.ui.KeyPicker
import com.intrada.android.ui.StagedExercise
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.sendFromForm
import com.intrada.shared.Accidental
import com.intrada.shared.Event
import com.intrada.shared.FormErrorField
import com.intrada.shared.FormErrorTarget
import com.intrada.shared.ItemEvent
import com.intrada.shared.Key
import com.intrada.shared.Letter
import com.intrada.shared.Modality
import kotlinx.coroutines.test.StandardTestDispatcher
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
class ItemFormSnapshotTest {
    @Test
    fun add() {
        captureRoboImage("src/test/snapshots/item-form-add.png") {
            ItemFormScreen(ItemFormState(), ItemFormMode.ADD, onCancel = {}, onConfirm = {})
        }
    }

    @Test
    fun addRefused() = runTest {
        val store = startedStore()
        val form =
            ItemFormState().apply {
                title = "Arpeggios"
                bpm = "999"
            }
        form.formError = store.sendFromForm(form.addEvent())
        captureRoboImage("src/test/snapshots/item-form-add-refused.png") {
            ItemFormScreen(form, ItemFormMode.ADD, onCancel = {}, onConfirm = {})
        }
    }

    @Test
    fun editAnExercise() = runTest {
        val store = startedStore()
        store.send(Event.Item(ItemEvent.Add(Fixtures.scales)))
        store.settle()
        val form = ItemFormState.of(store.libraryRows.value.single())
        captureRoboImage("src/test/snapshots/item-form-edit-exercise.png") {
            ItemFormScreen(form, ItemFormMode.EDIT, onCancel = {}, onConfirm = {})
        }
    }

    @Test
    fun keyWheelOpen() {
        captureRoboImage("src/test/snapshots/item-form-key-wheel.png") {
            Column(Modifier.background(IntradaColor.paperTop).padding(IntradaSpacing.card)) {
                Column(Modifier.cardSurface()) {
                    KeyPicker(
                        Key(Letter.G, Accidental.FLAT, Modality.MAJOR),
                        onKey = {},
                        initiallyExpanded = true,
                    )
                }
            }
        }
    }

    @Test
    fun relatedExercisesWithARefusedRow() {
        val form =
            ItemFormState().apply {
                exercises.addAll(
                    listOf(
                        StagedExercise.Written("Shell voicings", null, "80"),
                        StagedExercise.Written("Guide tones", null, "999"),
                        StagedExercise.Chosen("hanon", "Hanon No. 1", null),
                    )
                )
                faultedExercise = FormErrorTarget.Exercise(1u, FormErrorField.TEMPO)
            }
        captureRoboImage("src/test/snapshots/item-form-related-exercises.png") {
            Column(Modifier.background(IntradaColor.paperTop).padding(IntradaSpacing.card)) {
                ItemFormExercises(form, library = emptyList())
            }
        }
    }

    private suspend fun TestScope.startedStore(): Store {
        val store =
            Store(
                LiveBridge(),
                InMemoryItemStore(emptyList()),
                this,
                StandardTestDispatcher(testScheduler),
                log = {},
            )
        store.send(Event.StartApp)
        store.settle()
        return store
    }
}
