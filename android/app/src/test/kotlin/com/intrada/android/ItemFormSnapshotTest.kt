package com.intrada.android

import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Store
import com.intrada.android.ui.ItemFormMode
import com.intrada.android.ui.ItemFormScreen
import com.intrada.android.ui.ItemFormState
import com.intrada.android.ui.sendFromForm
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
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
