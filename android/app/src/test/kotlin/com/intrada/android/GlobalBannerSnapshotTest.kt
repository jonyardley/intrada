package com.intrada.android

import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Density
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

    @Test
    fun bannersAtTheLargestFontScale() {
        captureRoboImage("src/test/snapshots/banner-largest-font.png") {
            val density = LocalDensity.current
            CompositionLocalProvider(
                LocalDensity provides Density(density.density, fontScale = LARGEST_FONT_SCALE)
            ) {
                Banners()
            }
        }
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

    private companion object {
        const val LARGEST_FONT_SCALE = 2f
    }
}
