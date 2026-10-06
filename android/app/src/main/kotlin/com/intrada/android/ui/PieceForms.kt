package com.intrada.android.ui

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.intrada.shared.BarsInput
import com.intrada.shared.LinkChange
import com.intrada.shared.LinkedExerciseView
import com.intrada.shared.SectionChange
import com.intrada.shared.SectionEdit
import com.intrada.shared.SectionKind
import com.intrada.shared.SectionView

// ── What the piece page's screens hold while the musician edits ──

class SectionFormState(val existing: SectionView?) {
    var name by mutableStateOf(existing?.name.orEmpty())
    var bars by mutableStateOf(existing?.barsText.orEmpty())
    var kind by mutableStateOf(existing?.kind ?: SectionKind.FORM)
    var bpm by mutableStateOf(existing?.targetBpm?.toString().orEmpty())
    var formError by mutableStateOf<String?>(null)
    var confirmingRemoval by mutableStateOf(false)

    fun saveChange(): SectionChange =
        SectionChange.Save(SectionEdit(existing?.id, name, BarsInput.Typed(bars), kind, bpm))
}

private val SectionView.barsText: String
    get() {
        val first = firstBar ?: return ""
        val last = lastBar
        return if (last == null || last == first) "$first" else "$first to $last"
    }

class LinkSectionsState(val exercise: LinkedExerciseView) {
    var wholePiece by mutableStateOf(exercise.wholePiece)
    var sectionIds by mutableStateOf(exercise.sections.map { it.id }.toSet())
    var formError by mutableStateOf<String?>(null)

    val unchanged: Boolean
        get() =
            wholePiece == exercise.wholePiece &&
                sectionIds == exercise.sections.map { it.id }.toSet()

    fun toggle(sectionId: String) {
        sectionIds = if (sectionId in sectionIds) sectionIds - sectionId else sectionIds + sectionId
    }

    fun change(): LinkChange = LinkChange.Set(exercise.id, wholePiece, sectionIds.toList())
}

class ExercisePickerState(val linked: Set<String>) {
    var chosen by mutableStateOf(linked)
    var formError by mutableStateOf<String?>(null)

    fun toggle(id: String) {
        chosen = if (id in chosen) chosen - id else chosen + id
    }
}
