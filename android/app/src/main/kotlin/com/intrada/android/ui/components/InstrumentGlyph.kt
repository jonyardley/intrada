package com.intrada.android.ui.components

import androidx.annotation.DrawableRes
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.Dp
import com.intrada.android.R
import com.intrada.android.ui.IntradaColor
import com.intrada.android.ui.IntradaGlyph
import com.intrada.shared.InstrumentIcon

@Composable
fun InstrumentGlyph(
    icon: InstrumentIcon,
    modifier: Modifier = Modifier,
    size: Dp = IntradaGlyph.tile,
    tint: Color = IntradaColor.ink,
) {
    Image(
        painterResource(icon.drawable),
        contentDescription = icon.accessibilityLabel,
        modifier = modifier.size(size),
        colorFilter = ColorFilter.tint(tint),
    )
}

@get:DrawableRes
private val InstrumentIcon.drawable: Int
    get() =
        when (this) {
            InstrumentIcon.PIANO -> R.drawable.instrument_piano
            InstrumentIcon.ACOUSTICGUITAR -> R.drawable.instrument_acoustic_guitar
            InstrumentIcon.ELECTRICGUITAR -> R.drawable.instrument_electric_guitar
            InstrumentIcon.VIOLIN -> R.drawable.instrument_violin
            InstrumentIcon.CELLO -> R.drawable.instrument_cello
            InstrumentIcon.VOICE -> R.drawable.instrument_voice
            InstrumentIcon.FLUTE -> R.drawable.instrument_flute
            InstrumentIcon.CLARINET -> R.drawable.instrument_clarinet
            InstrumentIcon.SAXOPHONE -> R.drawable.instrument_saxophone
            InstrumentIcon.TRUMPET -> R.drawable.instrument_trumpet
            InstrumentIcon.DRUMS -> R.drawable.instrument_drums
            InstrumentIcon.HARP -> R.drawable.instrument_harp
            InstrumentIcon.OTHER -> R.drawable.instrument_other
        }

private val InstrumentIcon.accessibilityLabel: String
    get() =
        when (this) {
            InstrumentIcon.PIANO -> "Piano and keys"
            InstrumentIcon.ACOUSTICGUITAR -> "Acoustic guitar"
            InstrumentIcon.ELECTRICGUITAR -> "Electric guitar and bass"
            InstrumentIcon.VIOLIN -> "Violin and viola"
            InstrumentIcon.CELLO -> "Cello and double bass"
            InstrumentIcon.VOICE -> "Voice"
            InstrumentIcon.FLUTE -> "Flute"
            InstrumentIcon.CLARINET -> "Clarinet and oboe"
            InstrumentIcon.SAXOPHONE -> "Saxophone"
            InstrumentIcon.TRUMPET -> "Trumpet and brass"
            InstrumentIcon.DRUMS -> "Drums and percussion"
            InstrumentIcon.HARP -> "Harp"
            InstrumentIcon.OTHER -> "Plain note"
        }
