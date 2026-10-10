package com.intrada.android

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.ui.IntradaColor
import com.intrada.android.ui.IntradaSpacing
import com.intrada.android.ui.StartHereCard
import com.intrada.shared.FirstRunView
import com.intrada.shared.HighlighterColour
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = RobolectricDeviceQualifiers.Pixel7)
class StartHereSnapshotTest {
    @Test fun startHereAdd() = card("src/test/snapshots/start-here-add.png", progress())

    @Test
    fun startHereBuild() = card("src/test/snapshots/start-here-build.png", progress(added = true))

    @Test
    fun startHerePlay() =
        card("src/test/snapshots/start-here-play.png", progress(added = true, built = true))

    @Test
    fun startHereMark() =
        card(
            "src/test/snapshots/start-here-mark.png",
            progress(added = true, built = true, played = true),
        )

    private fun card(path: String, progress: FirstRunView) =
        captureRoboImage(path) {
            CardOnPaper {
                StartHereCard(
                    progress,
                    IntradaColor.marker(HighlighterColour.BUTTER),
                    onAdd = {},
                    onStartSession = {},
                )
            }
        }

    @Composable
    private fun CardOnPaper(content: @Composable () -> Unit) {
        Box(Modifier.width(360.dp).background(IntradaColor.paperTop).padding(IntradaSpacing.card)) {
            content()
        }
    }

    private fun progress(added: Boolean = false, built: Boolean = false, played: Boolean = false) =
        FirstRunView(
            showsWelcome = false,
            showsStartHere = true,
            added = added,
            built = built,
            played = played,
            marked = false,
            firstItemTitle = "Clair de Lune".takeIf { added },
        )
}
