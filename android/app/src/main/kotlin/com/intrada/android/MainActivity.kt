package com.intrada.android

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.viewModels
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Store
import com.intrada.android.core.withIds
import com.intrada.android.ui.LibraryScreen
import com.intrada.shared.Event
import java.util.TimeZone

class StoreHolder : ViewModel() {
    val store = Store(LiveBridge(), InMemoryItemStore(), viewModelScope)
    var started = false
}

class MainActivity : ComponentActivity() {
    private val holder: StoreHolder by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        val store = holder.store
        if (!holder.started) {
            holder.started = true
            val offsetMinutes = TimeZone.getDefault().getOffset(System.currentTimeMillis()) / 60_000
            store.send(Event.SetUtcOffset(offsetMinutes))
            // No StartApp in seed mode, so hydration cannot replace the sample set.
            store.send(
                if (intent.getBooleanExtra(EXTRA_SEED, false)) Event.LoadSampleData
                else Event.StartApp
            )
        }
        setContent {
            val viewModel by store.viewModel.collectAsState()
            val rows by store.libraryRows.collectAsState()
            LibraryScreen(rows.withIds(viewModel?.visibleIds.orEmpty()))
        }
    }

    companion object {
        const val EXTRA_SEED = "seed"
    }
}
