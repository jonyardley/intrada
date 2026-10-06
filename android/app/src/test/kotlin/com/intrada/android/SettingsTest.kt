package com.intrada.android

import android.content.Context
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Settings
import com.intrada.android.core.Store
import com.intrada.ffi.sessionBlobVersion
import com.intrada.shared.Event
import com.intrada.shared.SessionEvent
import java.io.File
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.RuntimeEnvironment
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class SettingsTest {
    private val prefs =
        RuntimeEnvironment.getApplication()
            .getSharedPreferences(Settings.PREFERENCES, Context.MODE_PRIVATE)

    // The bytes the core pins for the crash-recovery blob, which is what iOS writes: read from the
    // pin itself, so a re-pin cannot leave this test checking an old shape (#1345).
    @Test
    fun theBlobIosIsPinnedToWriteResumesThroughTheSlot() = runTest {
        val (version, bytes) = pinnedSessionBlob()
        assertEquals(sessionBlobVersion(), version)
        val settings = Settings(prefs)
        settings.sessionInProgress.write(bytes)
        assertNotEquals(
            String(bytes, Charsets.ISO_8859_1),
            prefs.getString(settings.sessionInProgress.key, null),
        )
        assertArrayEquals(bytes, settings.sessionInProgress.read())

        val store = store(settings)
        val session = checkNotNull(store.pendingSessionInProgress())
        store.send(Event.Session(SessionEvent.RecoverSession(session, "2026-09-03T09:10:00Z")))

        assertNotNull(store.viewModel.value?.activeSession)
    }

    @Test
    fun aPracticeAnOlderBuildSavedIsClearedAtLaunch() = runTest {
        val settings = Settings(prefs)
        val retired = settings.retiredSessionsInProgress.last()
        retired.write(byteArrayOf(1, 2, 3))
        val store = store(settings)

        store.loadRecoverableSession()

        assertNull(retired.read())
        assertNull(store.recoverableSession.value)
    }

    private fun TestScope.store(settings: Settings) =
        Store(
            LiveBridge(),
            InMemoryItemStore(),
            scope = this,
            io = StandardTestDispatcher(testScheduler),
            log = {},
            settings = settings,
        )

    private fun pinnedSessionBlob(): Pair<UInt, ByteArray> {
        val pin =
            generateSequence(File(checkNotNull(System.getProperty("user.dir"))).absoluteFile) {
                    it.parentFile
                }
                .map { File(it, "crates/intrada-core/src/domain/session/tests.rs") }
                .first { it.exists() }
                .readText()
        val version =
            checkNotNull(Regex("""const PINNED_BLOB_VERSION: u32 = (\d+);""").find(pin))
                .groupValues[1]
        val body =
            pin.substringAfter("const PINNED_ACTIVE_SESSION_HEX: &str = concat!(")
                .substringBefore(");")
        val hex = Regex("\"([0-9a-f]*)\"").findAll(body).joinToString("") { it.groupValues[1] }
        val bytes = hex.chunked(2).map { it.toInt(16).toByte() }.toByteArray()
        return version.toUInt() to bytes
    }
}
