package com.intrada.android.core

import android.util.Log
import com.intrada.ffi.InternalException
import com.intrada.shared.ActiveSession
import com.intrada.shared.AppEffect
import com.intrada.shared.Effect
import com.intrada.shared.Event
import com.intrada.shared.LibraryItemView
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.PersistenceOutput
import com.intrada.shared.RecognitionOutput
import com.intrada.shared.Request
import com.intrada.shared.SessionEvent
import com.intrada.shared.ViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * Every bridge call runs on [scope]'s dispatcher (Main in the app); only the store's own work goes
 * to [io].
 */
class Store(
    private val bridge: CoreBridge,
    private val itemStore: ItemStore,
    private val scope: CoroutineScope,
    private val io: CoroutineDispatcher = Dispatchers.IO,
    private val log: (String) -> Unit = { Log.i("intrada", it) },
    private val settings: Settings? = null,
    /** The database did not open, so nothing is kept; the shell warns (#2428). */
    val degraded: Boolean = false,
) {
    private val _viewModel = MutableStateFlow<ViewModel?>(null)
    val viewModel: StateFlow<ViewModel?> = _viewModel.asStateFlow()

    // Sent by the core only when a row changes, so a tap mid-practice does not replace them
    // (#1801).
    private val _libraryRows = MutableStateFlow<List<LibraryItemView>>(emptyList())
    val libraryRows: StateFlow<List<LibraryItemView>> = _libraryRows.asStateFlow()

    // The core panicked, or the bridge failed twice running: nothing after that can work, so
    // sends are refused without reaching the bridge and the screen shows a standing banner (#1946).
    private val _halted = MutableStateFlow(false)
    val halted: StateFlow<Boolean> = _halted.asStateFlow()
    private var consecutiveBridgeFailures = 0

    private var diskTail: Job? = null

    // A practice found at launch; Phase C offers to resume it (#962).
    private val _recoverableSession = MutableStateFlow<ActiveSession?>(null)
    val recoverableSession: StateFlow<ActiveSession?> = _recoverableSession.asStateFlow()

    init {
        _viewModel.value = bridged { bridge.view() }
    }

    fun send(event: Event) {
        process(bridged { bridge.update(event) }.orEmpty())
    }

    /** Waits until no disk job is queued, including jobs a resolve chains while this waits. */
    suspend fun settle() {
        while (true) {
            val tail = diskTail ?: return
            tail.join()
            if (diskTail === tail) diskTail = null
        }
    }

    private fun process(requests: List<Request>) {
        for (request in requests) {
            when (val effect = request.effect) {
                is Effect.Render -> bridged { bridge.view() }?.let { _viewModel.value = it }
                is Effect.App -> handleAppEffect(effect.value)
                is Effect.Persistence -> enqueueDiskJob(effect.value, request.id)
                is Effect.Recognition -> {
                    log("recognition is not built on Android yet; answering Failed")
                    process(
                        bridged { bridge.resolve(request.id, RecognitionOutput.Failed) }.orEmpty()
                    )
                }
            }
        }
    }

    // One job at a time, in the order the core asked: a load sent after a save must see the row.
    private fun enqueueDiskJob(operation: PersistenceOperation, id: UInt) {
        val previous = diskTail
        diskTail = scope.launch {
            previous?.join()
            val output =
                withContext(io) {
                    try {
                        itemStore.run(operation)
                    } catch (e: Exception) {
                        log("persistence failed: $e")
                        PersistenceOutput.Failed
                    }
                }
            process(bridged { bridge.resolve(id, output) }.orEmpty())
        }
    }

    private fun handleAppEffect(effect: AppEffect) {
        when (effect) {
            is AppEffect.LibraryChanged -> _libraryRows.value = effect.value
            AppEffect.ClearSessionInProgress -> {
                settings?.sessionInProgress?.clear()
                _recoverableSession.value = null
            }
            else ->
                if (settings?.keep(effect) != true) {
                    log("${effect::class.simpleName} is not handled on Android yet")
                }
        }
    }

    /** The saved sort, profile, practice defaults and welcome, sent to the core at launch. */
    fun restoreSettings() {
        settings?.restored()?.forEach(::send)
    }

    fun pendingSessionInProgress(): ActiveSession? = settings?.pendingSessionInProgress()

    /**
     * A practice saved by an older build has a shape this one cannot read, so it is never half
     * restored: its key goes and the core says so (#2246).
     */
    fun loadRecoverableSession() {
        val saved = settings ?: return
        if (_viewModel.value?.offersRecovery != true) return
        val retired = saved.retiredSessionsInProgress.filter { it.read() != null }
        if (retired.isNotEmpty()) {
            retired.forEach { it.clear() }
            send(Event.Session(SessionEvent.RetiredSessionFound))
        }
        _recoverableSession.value = pendingSessionInProgress()
    }

    // UniFFI raises a Rust panic as InternalException; CoreException and a failed decode are not.
    private fun <T> bridged(work: () -> T): T? {
        if (_halted.value) return null
        return try {
            work().also { consecutiveBridgeFailures = 0 }
        } catch (e: InternalException) {
            failed("core panic: $e", panicked = true)
        } catch (e: Exception) {
            failed("bridge failed: $e", panicked = false)
        }
    }

    private fun failed(message: String, panicked: Boolean): Nothing? {
        consecutiveBridgeFailures += 1
        log(message)
        if (panicked || consecutiveBridgeFailures >= 2) _halted.value = true
        return null
    }

    companion object {
        const val HALTED_MESSAGE =
            "The app has stopped responding · close and reopen it to carry on."
    }
}

/** The rows the Library filter leaves showing, in the core's order (#1998). */
fun List<LibraryItemView>.withIds(ids: List<String>): List<LibraryItemView> {
    val byId = associateBy { it.id }
    return ids.mapNotNull(byId::get)
}
