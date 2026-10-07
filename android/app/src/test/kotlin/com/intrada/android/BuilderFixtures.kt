package com.intrada.android

import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Store
import com.intrada.shared.BarsInput
import com.intrada.shared.BuildingSetlistView
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.SectionChange
import com.intrada.shared.SectionEdit
import com.intrada.shared.SectionKind
import com.intrada.shared.SessionEvent
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope

object BuilderFixtures {
    const val PIECE = "01J0000000000000000000ITEM"
    const val SATIE = "01J0000000000000000000SATI"
    const val HANON = "01J0000000000000000000HANO"
}

/** A library with a sectioned piece, its linked exercise and an exercise with variations. */
suspend fun TestScope.libraryStore(): Store {
    val store =
        Store(
            LiveBridge(),
            InMemoryItemStore(Fixtures.library),
            this,
            StandardTestDispatcher(testScheduler),
            log = {},
        )
    store.send(Event.StartApp)
    store.settle()
    listOf(
            SectionEdit(null, "A1", BarsInput.Typed("1 to 12"), SectionKind.FORM, ""),
            SectionEdit(null, "Run", BarsInput.Typed("19 to 20"), SectionKind.TROUBLESPOT, ""),
        )
        .forEach {
            store.send(
                Event.Item(ItemEvent.ChangeSection(BuilderFixtures.PIECE, SectionChange.Save(it)))
            )
        }
    store.send(
        Event.Item(
            ItemEvent.ChoosePieceExercises(
                BuilderFixtures.PIECE,
                listOf(BuilderFixtures.HANON),
                emptyList(),
            )
        )
    )
    store.send(Event.Item(ItemEvent.Add(Fixtures.scales)))
    store.settle()
    return store
}

/** Building, with Clair de Lune's block (Hanon, then the piece) and Satie standing alone. */
suspend fun TestScope.buildingStore(): Store {
    val store = libraryStore()
    store.send(Event.Session(SessionEvent.StartBuilding))
    store.send(Event.Session(SessionEvent.AddToSetlist(BuilderFixtures.PIECE)))
    store.send(Event.Session(SessionEvent.AddToSetlist(BuilderFixtures.SATIE)))
    store.settle()
    return store
}

fun Store.setlist(): BuildingSetlistView =
    viewModel.value?.buildingSetlist ?: error("the core is not building a session")

fun Store.scalesId(): String = libraryRows.value.single { it.title == Fixtures.scales.title }.id

fun BuildingSetlistView.titles(): List<String> = entries.map { it.itemTitle }
