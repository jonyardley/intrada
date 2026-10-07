package com.intrada.android.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import java.time.Instant
import java.time.OffsetDateTime
import java.time.format.DateTimeParseException
import java.util.Locale
import kotlinx.coroutines.delay

private const val MILLIS = 1000L

object SessionClock {
    fun now(): String = Instant.now().toString()

    fun parse(stamp: String): Instant? =
        try {
            OffsetDateTime.parse(stamp).toInstant()
        } catch (e: DateTimeParseException) {
            null
        }

    fun secondsBetween(start: Instant, end: Instant): Long =
        (end.toEpochMilli() - start.toEpochMilli()) / MILLIS

    fun clockDisplay(seconds: Long): String {
        val total = seconds.coerceAtLeast(0)
        val hours = total / 3600
        val minutes = (total % 3600) / 60
        val secs = total % 60
        return if (hours > 0) String.format(Locale.ROOT, "%d:%02d:%02d", hours, minutes, secs)
        else String.format(Locale.ROOT, "%02d:%02d", minutes, secs)
    }
}

/** Tests pass a fixed [held] instant so the screen is deterministic. */
@Composable
fun rememberTicking(held: Instant?): Instant {
    var now by remember { mutableStateOf(held ?: Instant.now()) }
    LaunchedEffect(held) {
        if (held != null) {
            now = held
            return@LaunchedEffect
        }
        while (true) {
            now = Instant.now()
            delay(MILLIS - now.toEpochMilli() % MILLIS)
        }
    }
    return now
}
