package com.intrada.android.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.unit.dp
import com.intrada.android.core.Store
import com.intrada.android.core.withIds
import com.intrada.android.ui.components.bar
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.components.label
import com.intrada.shared.Event
import com.intrada.shared.LibraryItemView

@Composable
fun LibraryRoute(store: Store, modifier: Modifier = Modifier) {
    val viewModel by store.viewModel.collectAsState()
    val rows by store.libraryRows.collectAsState()
    val halted by store.halted.collectAsState()
    LibraryScreen(
        rows.withIds(viewModel?.visibleIds.orEmpty()),
        error = viewModel?.error,
        halted = halted,
        onDismissError = { store.send(Event.ClearError) },
        modifier = modifier,
    )
}

@Composable
fun LibraryScreen(
    rows: List<LibraryItemView>,
    error: String?,
    halted: Boolean,
    onDismissError: () -> Unit,
    modifier: Modifier = Modifier,
) {
    ScreenScaffold("Library", modifier) {
        Column(Modifier.fillMaxSize()) {
            if (halted) GlobalBanner(Store.HALTED_MESSAGE, tag = "banner.halted")
            if (error != null) GlobalBanner(error, tag = "banner.error", onDismiss = onDismissError)
            if (rows.isEmpty()) {
                BasicText(
                    "Pieces and exercises will live here.",
                    Modifier.padding(IntradaSpacing.card),
                    style = IntradaFont.body.copy(color = IntradaColor.inkSecondary),
                )
            } else {
                LazyColumn(
                    contentPadding = PaddingValues(IntradaSpacing.card),
                    verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
                ) {
                    items(rows, key = { it.id }) {
                        LibraryItemCard(it, Modifier.testTag("library.row"))
                    }
                }
            }
        }
    }
}

@Composable
fun LibraryItemCard(item: LibraryItemView, modifier: Modifier = Modifier) {
    Row(
        modifier.fillMaxWidth().height(IntrinsicSize.Min).cardSurface().clearAndSetSemantics {
            contentDescription = item.spokenLabel
        }
    ) {
        Box(Modifier.width(4.dp).fillMaxHeight().background(item.itemType.bar))
        Column(Modifier.padding(IntradaSpacing.card)) {
            BasicText(item.title, style = IntradaFont.cardTitle.copy(color = IntradaColor.ink))
            if (item.subtitle.isNotEmpty()) {
                Spacer(Modifier.height(3.dp))
                BasicText(
                    item.subtitle,
                    style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
                )
            }
        }
    }
}

// Priority, links, ladder, key and tempo stay out of the label until the card draws them (#2266).
private val LibraryItemView.spokenLabel: String
    get() = listOf(itemType.label, title, subtitle).filter { it.isNotEmpty() }.joinToString(", ")
