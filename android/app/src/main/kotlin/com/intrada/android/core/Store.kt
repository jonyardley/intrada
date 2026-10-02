package com.intrada.android.core

import android.util.Log
import com.intrada.shared.AppEffect
import com.intrada.shared.Effect
import com.intrada.shared.Event
import com.intrada.shared.LibraryItemView
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.PersistenceOutput
import com.intrada.shared.RecognitionOutput
import com.intrada.shared.Request
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
) {
    private val _viewModel = MutableStateFlow<ViewModel?>(null)
    val viewModel: StateFlow<ViewModel?> = _viewModel.asStateFlow()

    // Sent by the core only when a row changes, so a tap mid-practice does not replace them
    // (#1801).
    private val _libraryRows = MutableStateFlow<List<LibraryItemView>>(emptyList())
    val libraryRows: StateFlow<List<LibraryItemView>> = _libraryRows.asStateFlow()

    private var diskTail: Job? = null

    init {
        _viewModel.value = bridged { bridge.view() }
    }

    fun send(event: Event) {
        process(bridged { bridge.update(event) } ?: emptyList())
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
                        bridged { bridge.resolve(request.id, RecognitionOutput.Failed) }
                            ?: emptyList()
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
            process(bridged { bridge.resolve(id, output) } ?: emptyList())
        }
    }

    private fun handleAppEffect(effect: AppEffect) {
        when (effect) {
            is AppEffect.LibraryChanged -> _libraryRows.value = effect.value
            else -> log("${effect::class.simpleName} is not handled on Android yet")
        }
    }

    private fun <T> bridged(work: () -> T): T? =
        try {
            work()
        } catch (e: Exception) {
            log("bridge failed: $e")
            null
        }
}

/** The rows the Library filter leaves showing, in the core's order (#1998). */
fun List<LibraryItemView>.withIds(ids: List<String>): List<LibraryItemView> {
    val byId = associateBy { it.id }
    return ids.mapNotNull(byId::get)
}
