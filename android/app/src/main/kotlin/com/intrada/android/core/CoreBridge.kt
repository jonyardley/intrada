package com.intrada.android.core

import com.intrada.ffi.CoreFfi
import com.intrada.shared.Event
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
