package com.intrada.android

import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import androidx.compose.foundation.layout.padding
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.unit.dp
import androidx.core.graphics.createBitmap
import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.ui.LoadedPhoto
import com.intrada.android.ui.PhotoCardActions
import com.intrada.android.ui.PhotoCardContent
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = RobolectricDeviceQualifiers.Pixel7)
class PhotoCardSnapshotTest {
    @Test
    fun photoCard() {
        val page = createBitmap(WIDTH, HEIGHT)
        val canvas = Canvas(page)
        canvas.drawColor(Color.WHITE)
        val staff = Paint().apply { color = Color.DKGRAY }
        for (line in 1..STAVES * LINES) {
            canvas.drawRect(MARGIN, line * GAP, WIDTH - MARGIN, line * GAP + 2f, staff)
        }
        captureRoboImage("src/test/snapshots/piece-photo.png") {
            PhotoCardContent(
                PHOTO,
                LoadedPhoto(PHOTO, page.asImageBitmap()),
                failure = null,
                PhotoCardActions({}, {}),
                Modifier.padding(16.dp),
            )
        }
    }

    private companion object {
        const val PHOTO = "01J0000000000000000000PHOT"
        const val WIDTH = 600
        const val HEIGHT = 840
        const val STAVES = 4
        const val LINES = 5
        const val GAP = 30f
        const val MARGIN = 40f
    }
}
