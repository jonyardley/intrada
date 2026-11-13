package com.intrada.android

import android.content.Context
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Settings
import com.intrada.android.core.Store
import com.intrada.ffi.sessionBlobVersion
import com.intrada.shared.ActiveSession
import com.intrada.shared.AppEffect
import com.intrada.shared.ClickStart
import com.intrada.shared.Event
import com.intrada.shared.FirstRunEvent
import com.intrada.shared.LibrarySort
import com.intrada.shared.PracticeDefaults
import com.intrada.shared.PracticeDefaultsEvent
import com.intrada.shared.SessionEvent
import com.intrada.shared.SortDirection
import com.intrada.shared.SortField
import java.io.File
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
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
    private val practice =
        RuntimeEnvironment.getApplication()
            .getSharedPreferences(Settings.PRACTICE_PREFERENCES, Context.MODE_PRIVATE)

    // The bytes the core pins for the crash-recovery blob, which is what iOS writes: read from the
    // pin itself, so a re-pin cannot leave this test checking an old shape (#1345).
    @Test
    fun theBlobIosIsPinnedToWriteResumesThroughTheSlot() = runTest {
        val (version, bytes) = pinnedSessionBlob()
        assertEquals(sessionBlobVersion(), version)
        val settings = Settings(prefs, practice)
        settings.sessionInProgress.write(bytes)
        assertNotEquals(
            String(bytes, Charsets.ISO_8859_1),
            practice.getString(settings.sessionInProgress.key, null),
        )
        assertArrayEquals(bytes, settings.sessionInProgress.read())

        val store = store(settings)
        val session = checkNotNull(store.pendingSessionInProgress())
        store.send(Event.Session(SessionEvent.RecoverSession(session, "2026-09-03T09:10:00Z")))

        assertNotNull(store.viewModel.value?.activeSession)
    }

    // The core owns both versions, and the keys are the iPhone's (#2216, #2791).
    @Test
    fun thePracticeDefaultsKeysFollowTheCore() {
        val settings = Settings(prefs, practice)
        assertEquals("intrada.practice-defaults.v3", settings.practiceDefaults.key)
        assertEquals("intrada.practice-defaults.v2", settings.previousPracticeDefaults.key)
    }

    @Test
    fun versionTwoDefaultsAreHandedToTheCoreAndKeptUntilSent() {
        val settings = Settings(prefs, practice)
        settings.previousPracticeDefaults.write(VERSION_TWO_DEFAULTS)

        val restored = settings.restored().filterIsInstance<Event.PracticeDefaults>()

        assertEquals(
            listOf(PracticeDefaultsEvent.Carried(VERSION_TWO_DEFAULTS.map { it.toUByte() })),
            restored.map { it.value },
        )
        assertNotNull("cleared only after the send", settings.previousPracticeDefaults.read())
    }

    @Test
    fun versionTwoDefaultsAreNotCarriedOnceVersionThreeIsSaved() {
        val settings = Settings(prefs, practice)
        val kept = PracticeDefaults(ClickStart.EVERYBEAT, 30.toUShort())
        settings.practiceDefaults.write(kept.bincodeSerialize())
        settings.previousPracticeDefaults.write(VERSION_TWO_DEFAULTS)

        val restored = settings.restored().filterIsInstance<Event.PracticeDefaults>()

        assertEquals(listOf(PracticeDefaultsEvent.Loaded(kept)), restored.map { it.value })
    }

    @Test
    fun carriedDefaultsAreSavedUnderTheNewKeyAndTheOldKeyGoes() = runTest {
        val settings = Settings(prefs, practice)
        settings.previousPracticeDefaults.write(VERSION_TWO_DEFAULTS)

        store(settings).restoreSettings()

        val saved =
            PracticeDefaults.bincodeDeserialize(checkNotNull(settings.practiceDefaults.read()))
        assertEquals(PracticeDefaults(ClickStart.TWOANDFOUR, 25.toUShort()), saved)
        assertNull(settings.previousPracticeDefaults.read())
    }

    @Test
    fun aPracticeAnOlderBuildSavedIsClearedAtLaunch() = runTest {
        val settings = Settings(prefs, practice)
        val retired = settings.retiredSessionsInProgress.last()
        retired.write(byteArrayOf(1, 2, 3))
        val store = store(settings)

        store.loadRecoverableSession()

        assertNull(retired.read())
        assertNotNull(store.viewModel.value?.notice)
    }

    @Test
    fun aSortAndAWelcomeChosenOnceAreBackAtTheNextLaunch() = runTest {
        val title = LibrarySort(SortField.TITLE, SortDirection.ASCENDING)
        val first = store(Settings(prefs, practice))
        assertNotEquals(title, first.viewModel.value?.activeSort)
        first.send(Event.SetSort(title))
        first.send(Event.FirstRun(FirstRunEvent.SkipWelcome))
        assertNotNull(Settings(prefs, practice).firstRun.read())

        val next = store(Settings(prefs, practice))
        next.restoreSettings()

        assertEquals(title, next.viewModel.value?.activeSort)
    }

    @Test
    fun aPracticeInProgressStaysOutOfTheBackedUpFile() {
        val settings = Settings(prefs, practice)
        settings.sessionInProgress.write(byteArrayOf(1, 2, 3))
        settings.retiredSessionsInProgress.last().write(byteArrayOf(4))
        settings.librarySort.write(byteArrayOf(5))

        assertEquals(setOf(settings.librarySort.key), prefs.all.keys)
    }

    @Test
    fun aPracticeIsKeptUntilTheCoreClearsIt() = runTest {
        val settings = Settings(prefs, practice)
        val bytes = pinnedSessionBlob().second
        val session = ActiveSession.bincodeDeserialize(bytes)
        // Two optional strings swapped in the generated field list would still round trip (#2582).
        assertEquals("evenness", session.entries.first().plan.intention)
        assertEquals("g1", session.entries.first().plan.groupId)
        assertTrue(settings.keep(AppEffect.SaveSessionInProgress(session)))
        assertArrayEquals(bytes, settings.sessionInProgress.read())
        val store = store(settings)

        // The core drops an empty practice rather than resume it, which is a clear the shell obeys.
        val empty = session.copy(entries = emptyList())
        store.send(Event.Session(SessionEvent.RecoverSession(empty, "2026-09-03T09:10:00Z")))

        assertNull(settings.sessionInProgress.read())
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

    private companion object {
        // Repetition target 5, the click on 2 and 4, 25 minutes: the core's version 2 pin.
        val VERSION_TWO_DEFAULTS = byteArrayOf(0x05, 0x01, 0x00, 0x00, 0x00, 0x01, 0x19, 0x00)
    }
}
