package com.intrada.android.core

import com.intrada.ffi.CoreException
import com.intrada.ffi.CoreFfi
import com.intrada.ffi.InternalException
import com.intrada.ffi.StoreFfi
import com.intrada.ffi.StoreFfiInterface
import com.intrada.shared.Event
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.PersistenceOutput
import com.intrada.shared.RecognitionOutput
import com.intrada.shared.Request
import com.intrada.shared.Requests
import com.intrada.shared.ViewModel

interface CoreBridge {
    fun update(event: Event): List<Request>

    fun resolve(id: UInt, output: PersistenceOutput): List<Request>

    fun resolve(id: UInt, output: RecognitionOutput): List<Request>

    fun view(): ViewModel
}

/** The sole owner of bincode and the UniFFI core handle. */
class LiveBridge : CoreBridge {
    private val core = CoreFfi()

    override fun update(event: Event): List<Request> =
        Requests.bincodeDeserialize(core.update(event.bincodeSerialize())).value

    override fun resolve(id: UInt, output: PersistenceOutput): List<Request> =
        Requests.bincodeDeserialize(core.resolve(id, output.bincodeSerialize())).value

    override fun resolve(id: UInt, output: RecognitionOutput): List<Request> =
        Requests.bincodeDeserialize(core.resolve(id, output.bincodeSerialize())).value

    override fun view(): ViewModel = ViewModel.bincodeDeserialize(core.view())
}

/**
 * The shared Rust store (#2421): the operation's bytes in, the output's bytes out, so the shell
 * never reads a row. Calls arrive one at a time off the main thread.
 */
class SharedItemStore(
    private val store: StoreFfiInterface,
    private val log: (String) -> Unit,
    private val reporter: Reporter,
) : ItemStore {
    override fun run(operation: PersistenceOperation): PersistenceOutput {
        val answer = store.handle(operation.bincodeSerialize())
        answer.unreadable.forEach {
            log("store could not read: $it")
            reporter.report(UnreadableStoredValue(it), DECODE_CONTEXT)
        }
        answer.error?.let { throw StoreFailure(it) }
        return PersistenceOutput.bincodeDeserialize(answer.output)
    }

    companion object {
        const val DECODE_CONTEXT = "LibraryStore decode"

        /**
         * A file that will not open, or a migration that panics on it, falls back to memory so the
         * app still launches; `degraded` says nothing will be kept.
         */
        fun open(
            path: String,
            log: (String) -> Unit,
            reporter: Reporter = SentryReporter,
        ): Opened =
            try {
                Opened(SharedItemStore(StoreFfi.open(path), log, reporter), degraded = false)
            } catch (e: CoreException) {
                inMemory(e, "store did not open, keeping nothing", log, reporter)
            } catch (e: InternalException) {
                inMemory(e, "store panicked opening, keeping nothing", log, reporter)
            }

        private fun inMemory(
            error: Exception,
            reason: String,
            log: (String) -> Unit,
            reporter: Reporter,
        ): Opened {
            log("$reason: $error")
            reporter.report(error, "store-open")
            return Opened(SharedItemStore(StoreFfi.inMemory(), log, reporter), degraded = true)
        }
    }

    class Opened(val store: SharedItemStore, val degraded: Boolean)
}

class StoreFailure(reason: String) : Exception(reason)

class UnreadableStoredValue(description: String) : Exception(description)
