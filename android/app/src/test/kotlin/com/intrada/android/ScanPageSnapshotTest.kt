package com.intrada.android

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.ui.Modifier
import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.ui.FieldMark
import com.intrada.android.ui.IntradaColor
import com.intrada.android.ui.IntradaSpacing
import com.intrada.android.ui.ScanPageCard
import com.intrada.android.ui.components.cardSurface
import com.intrada.shared.PhotoRecognitionStatus
import com.intrada.shared.PhotoRecognitionView
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = RobolectricDeviceQualifiers.Pixel7)
class ScanPageSnapshotTest {
    @Test
    fun states() {
        captureRoboImage("src/test/snapshots/scan-page-states.png") {
            Column(
                Modifier.background(IntradaColor.paperTop).padding(IntradaSpacing.card),
                verticalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
            ) {
                ScanPageCard(view(PhotoRecognitionStatus.IDLE, null), null, null, {}, {})
                ScanPageCard(view(PhotoRecognitionStatus.READING, PHOTO), null, null, {}, {})
                ScanPageCard(view(PhotoRecognitionStatus.FAILED, PHOTO), null, null, {}, {})
                ScanPageCard(
                    view(PhotoRecognitionStatus.READY, PHOTO, readNothing = true),
                    null,
                    "Couldn't save the photo. Try again.",
                    {},
                    {},
                )
                Column(Modifier.cardSurface()) {
                    FieldMark(weak = false)
                    FieldMark(weak = true)
                }
            }
        }
    }

    private fun view(
        status: PhotoRecognitionStatus,
        photoId: String?,
        readNothing: Boolean = false,
    ) = PhotoRecognitionView(status, photoId, null, readNothing)

    private companion object {
        const val PHOTO = "01J9Z3ZQ5V8Y6X4W2T0R8P6N4M"
    }
}
