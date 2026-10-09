package com.intrada.android

import android.graphics.Rect
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.PageReader
import com.intrada.android.core.Store
import com.intrada.android.core.Ulid
import com.intrada.android.core.recognisedLine
import com.intrada.android.ui.ItemFormState
import com.intrada.ffi.FormReadField
import com.intrada.shared.DraftSource
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.PageReading
import com.intrada.shared.PhotoDraft
import com.intrada.shared.PhotoRecognitionStatus
import com.intrada.shared.RecognisedLine
import com.intrada.shared.RecognitionOutput
import com.intrada.shared.TempoDraftField
import com.intrada.shared.TextDraftField
import java.io.IOException
import kotlin.math.roundToInt
import kotlin.random.Random
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class PageReadTest {
    @Test
    fun aMintedIdIsOneTheCoreAccepts() = runTest {
        val store = store(PageReader { RecognitionOutput.Failed })
        val photoId = Ulid.generate()

        store.send(Event.Item(ItemEvent.ReadPhoto(photoId)))

        val view = checkNotNull(store.viewModel.value)
        assertNull(view.error)
        assertEquals(photoId, view.photoRecognition.photoId)
        assertEquals(PhotoRecognitionStatus.READING, view.photoRecognition.status)
    }

    @Test
    fun ulidsAreTwentySixCrockfordCharactersInTimeOrder() {
        assertEquals("00000000000000000000000000", Ulid.generate(0, ZeroRandom))
        assertEquals("0000000001" + "0".repeat(16), Ulid.generate(1, ZeroRandom))
        val earlier = Ulid.generate(1_760_000_000_000)
        val later = Ulid.generate(1_760_000_000_001)
        assertTrue(earlier < later)
        assertTrue(Ulid.isValid(later))
        assertFalse(Ulid.isValid("0000000000000000000000000I"))
    }

    @Test
    fun aReadPageReachesTheCoreAsReady() = runTest {
        val title = RecognisedLine("Clair de Lune", 0.2f, 0.1f, 0.6f, 0.05f, 0.95f)
        val store = store(PageReader { RecognitionOutput.Page(PageReading(listOf(title), null)) })

        store.send(Event.Item(ItemEvent.ReadPhoto(Ulid.generate())))
        testScheduler.advanceUntilIdle()

        val recognition = checkNotNull(store.viewModel.value).photoRecognition
        assertEquals(PhotoRecognitionStatus.READY, recognition.status)
        assertEquals("Clair de Lune", recognition.draft?.title?.value)
    }

    @Test
    fun aReaderThatThrowsShowsTheReadAsFailed() = runTest {
        val store = store(PageReader { throw IOException("no model") })

        store.send(Event.Item(ItemEvent.ReadPhoto(Ulid.generate())))
        testScheduler.advanceUntilIdle()

        val view = checkNotNull(store.viewModel.value)
        assertEquals(PhotoRecognitionStatus.FAILED, view.photoRecognition.status)
        assertFalse(store.halted.value)
    }

    @Test
    fun pixelBoxesBecomeTopLeftFractionsClampedToThePage() {
        val line = recognisedLine("Allegro", Rect(100, 50, 300, 90), 400, 1000, 0.8f)
        assertEquals(
            listOf(0.25f, 0.05f, 0.5f, 0.04f),
            listOf(line.x, line.y, line.width, line.height).map {
                (it * 1000).roundToInt() / 1000f
            },
        )

        val overhang = recognisedLine("Op. 10", Rect(-20, 980, 420, 1010), 400, 1000, 0.5f)
        assertEquals(0f, overhang.x)
        assertEquals(1f, overhang.width)
        assertEquals(0.02f, overhang.height, 0.0001f)
    }

    @Test
    fun aFillMarksTheFieldsItWroteAndTypingTakesTheMarkOff() {
        val form = ItemFormState()
        form.fill(
            PhotoDraft(
                title = TextDraftField("Clair de Lune", DraftSource.RECOGNISED, 0.9f, false),
                composer = TextDraftField("Debussy", DraftSource.RECOGNISED, 0.3f, true),
                tempo =
                    TempoDraftField(
                        com.intrada.shared.Tempo("Andante", null),
                        DraftSource.RECOGNISED,
                        0.9f,
                        false,
                    ),
            )
        )

        assertEquals("Clair de Lune", form.title)
        assertEquals("Debussy", form.composer)
        assertEquals("Andante", form.marking)
        assertEquals(false, form.readFrom[FormReadField.TITLE])
        assertEquals(true, form.readFrom[FormReadField.COMPOSER])

        form.title = "Clair de lune"

        assertNull(form.readFrom[FormReadField.TITLE])
        assertEquals(true, form.readFrom[FormReadField.COMPOSER])
    }

    @Test
    fun theReadPhotoTravelsOnTheNewPiece() {
        val form =
            ItemFormState().apply {
                title = "Clair de Lune"
                photoId = "01J9Z3ZQ5V8Y6X4W2T0R8P6N4M"
            }
        val added = checkNotNull(form.addEvent() as? Event.Item)
        val piece = checkNotNull(added.value as? ItemEvent.Add).value
        assertEquals("01J9Z3ZQ5V8Y6X4W2T0R8P6N4M", piece.photoId)
    }

    private fun TestScope.store(reader: PageReader) =
        Store(
            LiveBridge(),
            InMemoryItemStore(),
            scope = this,
            io = StandardTestDispatcher(testScheduler),
            log = {},
            pageReader = reader,
        )

    private object ZeroRandom : Random() {
        override fun nextBits(bitCount: Int) = 0
    }
}
