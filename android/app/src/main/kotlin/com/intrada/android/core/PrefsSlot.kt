package com.intrada.android.core

import android.content.SharedPreferences
import androidx.core.content.edit
import com.intrada.ffi.firstRunBlobVersion
import com.intrada.ffi.librarySortBlobVersion
import com.intrada.ffi.profileBlobVersion
import com.intrada.ffi.sessionBlobVersion
import com.intrada.shared.ActiveSession
import com.intrada.shared.AppEffect
import com.intrada.shared.Event
import com.intrada.shared.FirstRun
import com.intrada.shared.FirstRunEvent
import com.intrada.shared.LibrarySort
import com.intrada.shared.PracticeDefaults
import com.intrada.shared.PracticeDefaultsEvent
import com.intrada.shared.Profile
import com.intrada.shared.ProfileEvent
import com.novi.serde.DeserializationError
import com.novi.serde.SerializationError
import java.util.Base64

/**
 * One bincode blob, held as Base64 text because SharedPreferences has no byte type. Its key changes
 * whenever the blob's shape does (#1345).
 */
class PrefsSlot(
    val key: String,
    private val prefs: SharedPreferences,
    private val log: (String) -> Unit = {},
) {
    fun read(): ByteArray? {
        val text = prefs.getString(key, null) ?: return null
        return try {
            Base64.getDecoder().decode(text)
        } catch (e: IllegalArgumentException) {
            log("$key is not Base64: $e")
            null
        }
    }

    fun write(bytes: ByteArray) {
        prefs.edit { putString(key, Base64.getEncoder().encodeToString(bytes)) }
    }

    fun clear() {
        prefs.edit { remove(key) }
    }
}

/**
 * The singletons and the crash-recovery blob, under the iPhone's keys (#2421). The practice in
 * progress has its own file, which the backup leaves out so a reinstall never resumes it (#2433).
 */
class Settings(
    prefs: SharedPreferences,
    practice: SharedPreferences,
    private val log: (String) -> Unit = {},
) {
    val librarySort = PrefsSlot("intrada.library-sort.v${librarySortBlobVersion()}", prefs, log)
    val sessionInProgress = PrefsSlot(sessionKey(sessionBlobVersion()), practice, log)
    val profile = PrefsSlot("intrada.profile.v${profileBlobVersion()}", prefs, log)
    // Its own blob, never fields on the profile, so no profile is lost to a failed decode (#1915).
    val practiceDefaults = PrefsSlot("intrada.practice-defaults.v2", prefs, log)
    val firstRun = PrefsSlot("intrada.first-run.v${firstRunBlobVersion()}", prefs, log)

    /** Practices saved by older builds, whose shape this one cannot read (#2246). */
    val retiredSessionsInProgress: List<PrefsSlot> =
        (1u until sessionBlobVersion()).map { PrefsSlot(sessionKey(it), practice, log) }

    /** Writes a save effect's blob; false for an effect that is not one. */
    fun keep(effect: AppEffect): Boolean {
        val (slot, encode) =
            when (effect) {
                is AppEffect.SaveLibrarySort -> librarySort to effect.value::bincodeSerialize
                is AppEffect.SaveSessionInProgress ->
                    sessionInProgress to effect.value::bincodeSerialize
                is AppEffect.SaveProfile -> profile to effect.value::bincodeSerialize
                is AppEffect.SavePracticeDefaults ->
                    practiceDefaults to effect.value::bincodeSerialize
                is AppEffect.SaveFirstRun -> firstRun to effect.value::bincodeSerialize
                else -> return false
            }
        try {
            slot.write(encode())
        } catch (e: SerializationError) {
            log("${slot.key} did not encode: $e")
        }
        return true
    }

    /** What was saved, as the events that hand it back to the core at launch. */
    fun restored(): List<Event> =
        listOfNotNull(
            decoded(librarySort) { LibrarySort.bincodeDeserialize(it) }?.let { Event.SetSort(it) },
            decoded(profile) { Profile.bincodeDeserialize(it) }
                ?.let { Event.Profile(ProfileEvent.Loaded(it)) },
            decoded(practiceDefaults) { PracticeDefaults.bincodeDeserialize(it) }
                ?.let { Event.PracticeDefaults(PracticeDefaultsEvent.Loaded(it)) },
            decoded(firstRun) { FirstRun.bincodeDeserialize(it) }
                ?.let { Event.FirstRun(FirstRunEvent.Loaded(it)) },
        )

    fun pendingSessionInProgress(): ActiveSession? =
        decoded(sessionInProgress) { ActiveSession.bincodeDeserialize(it) }

    private fun <T> decoded(slot: PrefsSlot, decode: (ByteArray) -> T): T? {
        val bytes = slot.read() ?: return null
        return try {
            decode(bytes)
        } catch (e: DeserializationError) {
            log("${slot.key} did not decode: $e")
            null
        }
    }

    companion object {
        const val PREFERENCES = "intrada"
        const val PRACTICE_PREFERENCES = "intrada.practice"

        fun sessionKey(version: UInt) = "intrada.session-in-progress.v$version"
    }
}
