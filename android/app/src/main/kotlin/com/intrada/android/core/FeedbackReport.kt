package com.intrada.android.core

import android.util.Log
import io.sentry.Attachment
import io.sentry.Hint
import io.sentry.Sentry
import io.sentry.protocol.Feedback

/** What Send feedback hands to Sentry (#598): the trimmed note, and the screen unless left out. */
class FeedbackReport private constructor(val message: String, val screenshot: ByteArray?) {
    companion object {
        fun of(note: String, screenshot: ByteArray?, includeScreenshot: Boolean): FeedbackReport? {
            val message = note.trim()
            if (message.isEmpty()) return null
            return FeedbackReport(message, screenshot.takeIf { includeScreenshot })
        }

        fun send(report: FeedbackReport) {
            // Builds without a DSN never start Sentry, so the note would vanish silently.
            if (!Sentry.isEnabled()) {
                Log.e("intrada", "feedback not sent: Sentry is off in this build")
                return
            }
            val hint =
                report.screenshot?.let {
                    Hint.withAttachment(Attachment(it, "screenshot.png", "image/png"))
                } ?: Hint()
            Sentry.feedback().capture(Feedback(report.message), hint)
        }
    }
}
