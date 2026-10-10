package com.intrada.android

import android.app.Application
import com.intrada.android.core.SentryStart

class IntradaApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        SentryStart.start(this)
    }
}
