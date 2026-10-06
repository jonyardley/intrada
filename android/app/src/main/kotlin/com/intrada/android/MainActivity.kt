package com.intrada.android

import android.app.Application
import android.content.Context
import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.viewModels
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Settings
import com.intrada.android.core.SharedItemStore
import com.intrada.android.core.Store
import com.intrada.android.ui.AppFrame
import com.intrada.shared.Event
import java.util.TimeZone

class StoreHolder(application: Application) : AndroidViewModel(application) {
    private var created: Store? = null
    var started = false

    // Seed mode keeps nothing, so the sample set never lands in the musician's notebook.
    fun store(seed: Boolean): Store =
        created ?: (if (seed) seeded() else kept()).also { created = it }

    private fun seeded() = Store(LiveBridge(), InMemoryItemStore(), viewModelScope)

    private fun kept(): Store {
        val app = getApplication<Application>()
        val log: (String) -> Unit = { Log.w("intrada", it) }
        val file = app.getDatabasePath(DATABASE)
        file.parentFile?.mkdirs()
        val opened = SharedItemStore.open(file.path, log)
        return Store(
            LiveBridge(),
            opened.store,
            viewModelScope,
            log = log,
            settings =
                Settings(
                    app.getSharedPreferences(Settings.PREFERENCES, Context.MODE_PRIVATE),
                    app.getSharedPreferences(Settings.PRACTICE_PREFERENCES, Context.MODE_PRIVATE),
                    log,
                ),
            degraded = opened.degraded,
        )
    }

    companion object {
        const val DATABASE = "intrada.sqlite"
    }
}

class MainActivity : ComponentActivity() {
    private val holder: StoreHolder by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        val seed = intent.getBooleanExtra(EXTRA_SEED, false)
        val store = holder.store(seed)
        if (!holder.started) {
            holder.started = true
            val offsetMinutes = TimeZone.getDefault().getOffset(System.currentTimeMillis()) / 60_000
            store.send(Event.SetUtcOffset(offsetMinutes))
            // No StartApp in seed mode, so hydration cannot replace the sample set.
            if (seed) {
                store.send(Event.LoadSampleData)
            } else {
                store.send(Event.StartApp)
                store.restoreSettings()
                store.loadRecoverableSession()
            }
        }
        setContent { AppFrame(store) }
    }

    companion object {
        const val EXTRA_SEED = "seed"
    }
}
