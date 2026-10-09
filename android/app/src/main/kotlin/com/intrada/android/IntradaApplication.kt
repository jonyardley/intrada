package com.intrada.android

import android.app.Application
import com.intrada.android.core.SentryStart

class IntradaApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        // Sentry starts before the database opens, or a store-open failure is dropped (#2058).
        SentryStart.start(this)
    }
}
