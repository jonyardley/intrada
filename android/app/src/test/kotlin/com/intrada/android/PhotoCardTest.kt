package com.intrada.android

import android.graphics.Bitmap
import android.graphics.Color
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.core.graphics.createBitmap
import androidx.test.core.app.ApplicationProvider
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.PhotoFiles
import com.intrada.android.core.Store
import com.intrada.android.core.Ulid
import com.intrada.android.ui.LoadedPhoto
import com.intrada.android.ui.PhotoCardActions
import com.intrada.android.ui.PhotoCardContent
import com.intrada.android.ui.PieceNavigation
import com.intrada.android.ui.PieceRoute
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36])
class PhotoCardTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun aPieceWithAPhotoShowsItAndATapOpensItFullScreen() = runTest {
        val store = pieceWithPhoto(written = true)
        compose.setContent { PieceRoute(store, PIECE, navigation) }

        compose.waitUntil { compose.onAllNodesWithTag("piece.photo").fetchSemanticsNodes().any() }
        compose.onNodeWithTag("piece.photo").performScrollTo().performClick()

        compose.onNodeWithTag("piece.photo.full").assertIsDisplayed()
    }

    @Test
    fun removingThePhotoAsksFirstThenClearsIt() = runTest {
        val store = pieceWithPhoto(written = true)
        compose.setContent { PieceRoute(store, PIECE, navigation) }

        compose.onNodeWithTag("piece.photo.remove").performScrollTo().performClick()
        assertEquals(1, store.libraryRows.value.count { it.photoId != null })
        compose.onNodeWithTag("piece.photo.removeConfirm.confirm").performClick()
        store.settle()

        assertNull(store.libraryRows.value.single { it.id == PIECE }.photoId)
    }

    @Test
    fun aPhotoWhoseFileHasGoneSaysSo() = runTest {
        val store = pieceWithPhoto(written = false)
        compose.setContent { PieceRoute(store, PIECE, navigation) }

        compose.waitUntil {
            compose.onAllNodesWithTag("piece.photo.missing").fetchSemanticsNodes().any()
        }
        compose.onNodeWithText("That photo is missing.").performScrollTo().assertIsDisplayed()
    }

    @Test
    fun changeOffersAScanWhenThePhoneCanScan() {
        var scanned = 0
        var chosen = 0
        compose.setContent {
            PhotoCardContent(
                PHOTO,
                LoadedPhoto(PHOTO, null),
                failure = null,
                PhotoCardActions({ chosen++ }, {}, onScan = { scanned++ }),
            )
        }

        compose.onNodeWithTag("piece.photo.choose").performClick()
        compose.onNodeWithTag("piece.photo.library").assertIsDisplayed()
        compose.onNodeWithTag("piece.photo.scan").performClick()

        assertEquals(1, scanned)
        assertEquals(0, chosen)
        compose.onNodeWithTag("piece.photo.scan").assertDoesNotExist()
    }

    @Test
    fun changeGoesStraightToThePhotosWhenThePhoneCannotScan() {
        var chosen = 0
        compose.setContent {
            PhotoCardContent(
                PHOTO,
                LoadedPhoto(PHOTO, null),
                failure = null,
                PhotoCardActions({ chosen++ }, {}),
            )
        }

        compose.onNodeWithTag("piece.photo.choose").performClick()

        assertEquals(1, chosen)
        compose.onNodeWithTag("piece.photo.scan").assertDoesNotExist()
    }

    private suspend fun TestScope.pieceWithPhoto(written: Boolean): Store {
        val photoId = Ulid.generate()
        if (written) {
            val file = PhotoFiles.of(ApplicationProvider.getApplicationContext()).file(photoId)
            file?.parentFile?.mkdirs()
            file?.outputStream()?.use {
                createBitmap(PIXELS, PIXELS)
                    .apply { eraseColor(Color.GRAY) }
                    .compress(Bitmap.CompressFormat.JPEG, QUALITY, it)
            }
        }
        val store =
            Store(
                LiveBridge(),
                InMemoryItemStore(Fixtures.library),
                this,
                StandardTestDispatcher(testScheduler),
                log = {},
            )
        store.send(Event.StartApp)
        store.settle()
        store.send(Event.Item(ItemEvent.SetPhoto(PIECE, photoId)))
        store.settle()
        return store
    }

    private val navigation = PieceNavigation({}, {}, {}, {}, {}, {})

    private companion object {
        const val PIECE = "01J0000000000000000000ITEM"
        const val PHOTO = "01J0000000000000000000PHOT"
        const val PIXELS = 40
        const val QUALITY = 80
    }
}
