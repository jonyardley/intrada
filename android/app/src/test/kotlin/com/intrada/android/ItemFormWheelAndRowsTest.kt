package com.intrada.android

import android.os.Bundle
import android.os.Parcel
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.saveable.SaverScope
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.test.assertContentDescriptionContains
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.junit4.StateRestorationTester
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextClearance
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.performTouchInput
import androidx.core.os.BundleCompat
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Store
import com.intrada.android.ui.ItemFormMode
import com.intrada.android.ui.ItemFormScreen
import com.intrada.android.ui.ItemFormState
import com.intrada.android.ui.LibraryAddRoute
import com.intrada.android.ui.LibraryEditRoute
import com.intrada.android.ui.VariationRow
import com.intrada.android.ui.VariationRowsCard
import com.intrada.shared.Accidental
import com.intrada.shared.CreateItem
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKind
import com.intrada.shared.Key
import com.intrada.shared.KeyEdit
import com.intrada.shared.Letter
import com.intrada.shared.Modality
import com.intrada.shared.ScoreHistoryEntry
import java.io.Serializable
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class ItemFormWheelAndRowsTest {
    @get:Rule val compose = createComposeRule()

    private val cMajor = Key(Letter.C, Accidental.NATURAL, Modality.MAJOR)
    private val gMajor = Key(Letter.G, Accidental.NATURAL, Modality.MAJOR)

    @Test
    fun aTappedSpokeIsTheKeyThePieceSavesWith() = runTest {
        val store = startedStore()
        compose.setContent { LibraryAddRoute(store, onDone = {}) }

        compose.onNodeWithTag("itemForm.title").performTextInput("Prelude")
        compose.onNodeWithTag("itemForm.key").performClick()
        compose.onNodeWithTag("itemForm.key.major.1").performScrollTo().performClick()
        compose.onNodeWithTag("itemForm.key.major.1").assertIsSelected()
        compose
            .onNodeWithTag("itemForm.key")
            .assertContentDescriptionContains("G major", substring = true)
        compose.onNodeWithTag("itemForm.confirm").performClick()
        store.settle()

        assertEquals(gMajor, store.libraryRows.value.single().key)
    }

    @Test
    fun anEditChangesTheKey() = runTest {
        val store = editing(cMajor)
        compose.onNodeWithTag("itemForm.key").performClick()
        compose.onNodeWithTag("itemForm.key.major.0").assertIsSelected()
        compose.onNodeWithTag("itemForm.key.major.1").performScrollTo().performClick()
        compose.onNodeWithTag("itemForm.confirm").performClick()
        store.settle()

        assertEquals(gMajor, store.libraryRows.value.single().key)
    }

    @Test
    fun aSecondTapFlipsTheSpelling() = runTest {
        val store = startedStore()
        compose.setContent { LibraryAddRoute(store, onDone = {}) }

        compose.onNodeWithTag("itemForm.title").performTextInput("Prelude")
        compose.onNodeWithTag("itemForm.key").performClick()
        val spoke = compose.onNodeWithTag("itemForm.key.major.6").performScrollTo()
        listOf("G flat major", "F sharp major", "G flat major", "F sharp major").forEach {
            spoke.performClick()
            compose
                .onNodeWithTag("itemForm.key")
                .assertContentDescriptionContains(it, substring = true)
        }
        compose.onNodeWithTag("itemForm.confirm").performClick()
        store.settle()

        assertEquals(
            Key(Letter.F, Accidental.SHARP, Modality.MAJOR),
            store.libraryRows.value.single().key,
        )
    }

    @Test
    fun aSecondTapFlipsTheMinorSpelling() = runTest {
        val store = startedStore()
        compose.setContent { LibraryAddRoute(store, onDone = {}) }

        compose.onNodeWithTag("itemForm.title").performTextInput("Prelude")
        compose.onNodeWithTag("itemForm.key").performClick()
        val spoke = compose.onNodeWithTag("itemForm.key.minor.6").performScrollTo()
        listOf("E flat minor", "D sharp minor").forEach {
            spoke.performClick()
            compose
                .onNodeWithTag("itemForm.key")
                .assertContentDescriptionContains(it, substring = true)
        }
        compose.onNodeWithTag("itemForm.confirm").performClick()
        store.settle()

        assertEquals(
            Key(Letter.D, Accidental.SHARP, Modality.MINOR),
            store.libraryRows.value.single().key,
        )
    }

    @Test
    fun anEditClearsTheKey() = runTest {
        val store = editing(cMajor)
        compose.onNodeWithTag("itemForm.key.clear").performClick()
        compose
            .onNodeWithTag("itemForm.key")
            .assertContentDescriptionContains("no key selected", substring = true)
        compose.onNodeWithTag("itemForm.confirm").performClick()
        store.settle()

        assertNull(store.libraryRows.value.single().key)
    }

    // The form sends an emptied composer or notes as a blank, which the core clears (#2417).
    @Test
    fun anEditClearsTheComposerAndNotes() = runTest {
        val store = editing(Fixtures.scales.copy(composer = "Czerny", notes = "Slowly"))
        compose.onNodeWithTag("itemForm.composer").performTextClearance()
        compose.onNodeWithTag("itemForm.notes").performScrollTo().performTextClearance()
        compose.onNodeWithTag("itemForm.confirm").performClick()
        store.settle()

        val row = store.libraryRows.value.single()
        assertEquals("", row.subtitle)
        assertNull(row.notes)
    }

    @Test
    fun aClearedKeyIsSentAsAClear() {
        val form = ItemFormState(ItemKind.PIECE).apply { title = "Prelude" }
        val edit = (form.editEvent("01J0000000000000000000ITEM") as? Event.Item)?.value
        assertEquals(KeyEdit.Clear, (edit as? ItemEvent.Edit)?.input?.key)
    }

    @Test
    fun rotatingMidFormKeepsEverything() = runTest {
        val store = startedStore()
        val restoration = StateRestorationTester(compose)
        restoration.setContent { LibraryAddRoute(store, onDone = {}) }

        compose.onNodeWithTag("itemForm.kind.exercise").performClick()
        compose.onNodeWithTag("itemForm.title").performTextInput("Arpeggios")
        compose.onNodeWithTag("itemForm.composer").performTextInput("Czerny")
        compose.onNodeWithTag("itemForm.key").performClick()
        compose.onNodeWithTag("itemForm.key.major.0").performScrollTo().performClick()
        compose
            .onNodeWithTag("itemForm.variation.input")
            .performScrollTo()
            .performTextInput("Legato")
        compose.onNodeWithTag("itemForm.variation.add").performClick()
        compose.onNodeWithTag("itemForm.variation.input").performTextInput("Stacc")

        restoration.emulateSavedInstanceStateRestore()

        compose.onNodeWithText("Arpeggios").assertExists()
        compose.onNodeWithText("Czerny").assertExists()
        compose
            .onNodeWithTag("itemForm.key")
            .assertContentDescriptionContains("C major", substring = true)
        compose.onNodeWithTag("itemForm.key.major.0").assertIsSelected()
        compose.onNodeWithText("Legato").assertExists()
        compose.onNodeWithText("Stacc").assertExists()
        compose.onNodeWithTag("itemForm.confirm").performClick()
        store.settle()
        val added = store.libraryRows.value.single()
        assertEquals(ItemKind.EXERCISE, added.itemType)
        assertEquals(cMajor, added.key)
        assertEquals(listOf("Legato"), added.variations.map { it.label })
    }

    // A real rotation writes the state into a Bundle, which the restoration tester never does.
    @Test
    fun theSavedFormFitsInABundleAndComesBackWhole() {
        val form =
            ItemFormState(ItemKind.EXERCISE).apply {
                title = "Scales"
                key = gMajor
                bpm = "90"
                tags.add("warm-up")
                variations.add(VariationRow("01J0000000000000000000VARI", "Slow", hasMarks = true))
                variations.add(VariationRow(null, "Swung"))
            }
        val saved = with(ItemFormState.Saver) { SaverScope { true }.save(form) }
        val parcel = Parcel.obtain()
        parcel.writeBundle(Bundle().apply { putSerializable("form", saved as? Serializable) })
        parcel.setDataPosition(0)
        val unparcelled =
            parcel.readBundle(javaClass.classLoader)?.let {
                BundleCompat.getSerializable(it, "form", ArrayList::class.java)
            }
        parcel.recycle()
        val back = ItemFormState.Saver.restore(checkNotNull(unparcelled))

        assertEquals("Scales", back?.title)
        assertEquals(gMajor, back?.key)
        assertEquals("90", back?.bpm)
        assertEquals(listOf("warm-up"), back?.tags?.toList())
        assertEquals(
            listOf(
                Triple("01J0000000000000000000VARI", "Slow", true),
                Triple(null, "Swung", false),
            ),
            back?.variations?.map { Triple(it.variantId, it.label, it.hasMarks) },
        )
        assertEquals(form.variations.map { it.id }, back?.variations?.map { it.id })
    }

    @Test
    fun aRowDraggedPastItsNeighbourSavesInItsNewPlace() = runTest {
        val store = startedStore()
        store.send(Event.Item(ItemEvent.Add(Fixtures.scales)))
        store.settle()
        val id = store.libraryRows.value.single().id
        compose.setContent { LibraryEditRoute(store, id, onDone = {}) }

        compose.onAllNodesWithTag("variationRow.reorder")[0].performScrollTo().performTouchInput {
            down(center)
            advanceEventTime(viewConfiguration.longPressTimeoutMillis + 100)
            repeat(10) { moveBy(Offset(0f, height * 0.15f)) }
            up()
        }
        compose.onNodeWithTag("itemForm.confirm").performClick()
        store.settle()

        assertEquals(
            listOf("Swung", "Slow"),
            store.libraryRows.value.single().variations.map { it.label },
        )
    }

    @Test
    fun talkBackMovesARowDown() {
        val rows = mutableStateListOf(VariationRow(null, "Slow"), VariationRow(null, "Swung"))
        compose.setContent { VariationRowsCard(rows) }

        val actions =
            compose
                .onAllNodesWithTag("variationRow.reorder")[0]
                .fetchSemanticsNode()
                .config[SemanticsActions.CustomActions]
        compose.runOnIdle { actions.single { it.label == "Move down" }.action() }

        assertEquals(listOf("Swung", "Slow"), rows.map { it.label })
    }

    @Test
    fun removingARowWithMarksAsksFirst() = runTest {
        val store = startedStore()
        store.send(Event.Item(ItemEvent.Add(Fixtures.scales)))
        store.settle()
        val item = store.libraryRows.value.single()
        val played = ScoreHistoryEntry("2026-10-01", 4u, "01J0000000000000000SESSION")
        val marked =
            item.copy(
                variations =
                    item.variations.mapIndexed { index, variation ->
                        if (index == 0) variation.copy(scoreHistory = listOf(played)) else variation
                    }
            )
        val form = ItemFormState.of(marked)
        compose.setContent {
            ItemFormScreen(form, ItemFormMode.EDIT, onCancel = {}, onConfirm = {})
        }

        compose.onAllNodesWithTag("itemForm.variation.remove")[1].performScrollTo().performClick()
        assertEquals(listOf("Slow"), form.variations.map { it.label })

        compose.onAllNodesWithTag("itemForm.variation.remove")[0].performScrollTo().performClick()
        compose.onNodeWithText("Its marks go with it.").assertExists()
        compose.onNodeWithTag("itemForm.variationRemoval.cancel").performClick()
        assertEquals(listOf("Slow"), form.variations.map { it.label })

        compose.onAllNodesWithTag("itemForm.variation.remove")[0].performClick()
        compose.onNodeWithTag("itemForm.variationRemoval.confirm").performClick()
        assertTrue(form.variations.isEmpty())
        assertFalse(compose.onAllNodesWithText("Its marks go with it.").fetchSemanticsNodes().any())
    }

    private suspend fun TestScope.editing(key: Key): Store =
        editing(Fixtures.scales.copy(key = key))

    private suspend fun TestScope.editing(item: CreateItem): Store {
        val store = startedStore()
        store.send(Event.Item(ItemEvent.Add(item)))
        store.settle()
        val id = store.libraryRows.value.single().id
        compose.setContent { LibraryEditRoute(store, id, onDone = {}) }
        return store
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
