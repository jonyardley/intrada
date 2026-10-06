package com.intrada.android

import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable
import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.ui.GlobalBanner
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = RobolectricDeviceQualifiers.Pixel7)
class GlobalBannerSnapshotTest {
    @Test
    fun banners() {
        captureRoboImage("src/test/snapshots/banner.png") { Banners() }
    }

    @Composable
    private fun Banners() {
        Column {
            GlobalBanner("Couldn't delete that item.", tag = "banner.error", onDismiss = {})
            GlobalBanner(
                "Storage unavailable · changes this session won't be saved.",
                tag = "banner.halted",
            )
        }
    }
}
