package com.intrada.android.ui

import androidx.compose.foundation.relocation.BringIntoViewRequester
import com.intrada.shared.FormErrorField
import com.intrada.shared.FormErrorTarget

/** One per refused save, so refusing the same field twice still scrolls to it. */
data class Refusal(val target: FormErrorTarget?, val seq: Int)

internal enum class FormAnchor {
    TITLE,
    COMPOSER,
    VARIATIONS,
    PRACTICE,
    NOTES,
    TAGS,
}

internal class FormAnchors {
    private val cards = FormAnchor.entries.associateWith { BringIntoViewRequester() }
    val row = BringIntoViewRequester()

    fun at(anchor: FormAnchor): BringIntoViewRequester = cards.getValue(anchor)

    suspend fun bringIntoView(target: FormErrorTarget?) {
        val requester =
            when (target) {
                is FormErrorTarget.Piece -> target.field.anchor?.let(cards::getValue)
                is FormErrorTarget.Exercise -> row
                else -> null
            }
        requester?.bringIntoView()
    }
}

private val FormErrorField.anchor: FormAnchor?
    get() =
        when (this) {
            FormErrorField.TITLE -> FormAnchor.TITLE
            FormErrorField.COMPOSER -> FormAnchor.COMPOSER
            FormErrorField.TEMPO -> FormAnchor.PRACTICE
            FormErrorField.NOTES -> FormAnchor.NOTES
            FormErrorField.TAGS -> FormAnchor.TAGS
            FormErrorField.VARIATIONS -> FormAnchor.VARIATIONS
            FormErrorField.SECTIONS -> null
        }
