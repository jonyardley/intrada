package com.intrada.android.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import com.intrada.android.core.Store
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.FormErrorBanner
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.LibraryItemView

@Composable
fun LinkSectionsRoute(
    store: Store,
    pieceId: String,
    exerciseId: String,
    onDone: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val rows by store.libraryRows.collectAsState()
    val piece = rows.firstOrNull { it.id == pieceId }
    val exercise = piece?.linkedExercises?.firstOrNull { it.id == exerciseId }
    if (piece == null || exercise == null) {
        MissingItem("Sections", NO_LONGER_THERE, modifier)
        return
    }
    val state = remember(pieceId, exerciseId) { LinkSectionsState(exercise) }
    var closing by remember { mutableStateOf(false) }
    LinkSectionsScreen(
        piece,
        state,
        onCancel = onDone,
        onDone = {
            if (!closing) {
                if (!state.unchanged) {
                    state.formError =
                        store.sendFromForm(
                            Event.Item(ItemEvent.ChangePieceLink(pieceId, state.change()))
                        )
                }
                closing = state.formError == null
                if (closing) onDone()
            }
        },
        modifier,
    )
}

@Composable
fun LinkSectionsScreen(
    piece: LibraryItemView,
    state: LinkSectionsState,
    onCancel: () -> Unit,
    onDone: () -> Unit,
    modifier: Modifier = Modifier,
) {
    ScreenScaffold(
        state.exercise.title,
        modifier,
        actions = {
            TextAction("Cancel", "linkSectionsSheet.cancel", onCancel)
            TextAction("Done", "linkSectionsSheet.done", onDone, emphasised = true)
        },
    ) {
        Column(
            Modifier.fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(IntradaSpacing.card),
            verticalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
        ) {
            state.formError?.let {
                FormErrorBanner(it, Modifier.testTag("linkSectionsSheet.error"))
            }
            Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact)) {
                FieldLabel("Practise it for")
                Column(Modifier.cardSurface()) {
                    TickRow(
                        "The whole piece",
                        caption = null,
                        chosen = state.wholePiece,
                        tag = "linkSectionsSheet.row",
                        onToggle = { state.wholePiece = !state.wholePiece },
                    )
                    piece.sections.forEach { section ->
                        HairlineDivider()
                        TickRow(
                            section.label,
                            section.barsCaption,
                            chosen = section.id in state.sectionIds,
                            tag = "linkSectionsSheet.row",
                            onToggle = { state.toggle(section.id) },
                        )
                    }
                }
            }
            BasicText(
                "Untick everything to take it off this piece.",
                style = IntradaFont.small.copy(color = IntradaColor.inkSecondary),
            )
        }
    }
}
