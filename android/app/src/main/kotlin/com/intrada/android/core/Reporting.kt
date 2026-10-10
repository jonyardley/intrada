package com.intrada.android.core

import android.content.Context
import android.os.Build
import android.util.Log
import com.intrada.android.BuildConfig
import com.intrada.shared.Event
import io.sentry.Breadcrumb
import io.sentry.Sentry
import io.sentry.android.core.SentryAndroid

/** What the store tells Sentry: failures it swallows, and the steps that led to them. */
interface Reporter {
    fun report(error: Throwable, context: String)

    fun step(event: Event)
}

object SentryReporter : Reporter {
    override fun report(error: Throwable, context: String) {
        Sentry.captureException(error) { scope ->
            scope.setTag("report_context", context)
            scope.fingerprint = listOf("{{ default }}", context)
        }
    }

    override fun step(event: Event) {
        Sentry.addBreadcrumb(Breadcrumb.user("event", stepName(event)))
    }

    /** Feedback sent from a shake names the screen the tester was on, as on iOS (#598). */
    fun screen(name: String) {
        Sentry.configureScope { it.setTag("screen", name) }
    }
}

/** The variant names only: a payload can carry a title or a note. */
fun stepName(event: Event): String =
    when (event) {
        is Event.Item -> "Item.${event.value::class.simpleName}"
        is Event.Session -> "Session.${event.value::class.simpleName}"
        is Event.Profile -> "Profile.${event.value::class.simpleName}"
        is Event.PracticeDefaults -> "PracticeDefaults.${event.value::class.simpleName}"
        is Event.FirstRun -> "FirstRun.${event.value::class.simpleName}"
        is Event.Variation -> "Variation.${event.value::class.simpleName}"
        else -> event::class.simpleName.orEmpty()
    }

/** Pinned rather than left to the SDK default, so the release lane composes the same name. */
fun releaseName(packageName: String?, versionName: String?, versionCode: Long?): String? {
    if (packageName.isNullOrEmpty() || versionName.isNullOrEmpty() || versionCode == null) {
        return null
    }
    return "$packageName@$versionName+$versionCode"
}

object SentryStart {
    private const val RELEASE_TRACE_RATE = 0.2

    fun start(context: Context, dsn: String = BuildConfig.SENTRY_DSN) {
        // Robolectric builds the Application too; a DSN in a developer's shell must not report
        // tests.
        if (!dsn.startsWith("https://") || Build.FINGERPRINT == "robolectric") return
        val info = runCatching {
            context.packageManager.getPackageInfo(context.packageName, 0)
        }
            .onFailure { Log.w("intrada", "no package info for the Sentry release: $it") }
            .getOrNull()
        SentryAndroid.init(context) { options ->
            options.dsn = dsn
            releaseName(context.packageName, info?.versionName, info?.longVersionCode)?.let {
                options.release = it
            }
            options.environment = if (BuildConfig.DEBUG) "development" else "production"
            options.tracesSampleRate = if (BuildConfig.DEBUG) 1.0 else RELEASE_TRACE_RATE
        }
    }
}
