package com.intrada.android.core

import com.intrada.shared.Item
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.PersistenceOutput
import com.intrada.shared.PracticeSession
import com.intrada.shared.StoredItem
import com.intrada.shared.StoredRecords
import com.intrada.shared.Variation

/** Answers the core's persistence operations: rows in, rows out, no decisions. */
interface ItemStore {
    fun run(operation: PersistenceOperation): PersistenceOutput
}

// Seed mode and tests: the app keeps its notebook in SharedItemStore (#2421).
class InMemoryItemStore(items: List<Item> = emptyList()) : ItemStore {
    private val items = LinkedHashMap<String, Item>().apply { items.forEach { put(it.id, it) } }
    private val sessions = LinkedHashMap<String, PracticeSession>()
    private val variations = LinkedHashMap<String, Variation>()

    @Synchronized
    override fun run(operation: PersistenceOperation): PersistenceOutput =
        when (operation) {
            PersistenceOperation.LoadItems -> PersistenceOutput.Items(items.values.toList())
            PersistenceOperation.LoadSessions ->
                PersistenceOutput.Sessions(sessions.values.toList())
            is PersistenceOperation.SaveItem -> ack { items[operation.value.id] = operation.value }
            is PersistenceOperation.SaveItems ->
                ack { operation.value.forEach { items[it.id] = it } }
            is PersistenceOperation.DeleteItem -> ack { items.remove(operation.id) }
            is PersistenceOperation.SaveSession ->
                ack { sessions[operation.value.id] = operation.value }
            PersistenceOperation.LoadVariations ->
                PersistenceOutput.Variations(variations.values.toList())
            is PersistenceOperation.SaveVariations ->
                ack { operation.value.forEach { variations[it.id] = it } }
            is PersistenceOperation.LoadRecords,
            PersistenceOperation.LoadAllRecords ->
                PersistenceOutput.Records(
                    StoredRecords(
                        items.values.map { StoredItem(it, null) },
                        variations.values.toList(),
                        sessions.values.toList(),
                    )
                )
            is PersistenceOperation.ApplyMerged ->
                ack {
                    operation.value.items.forEach {
                        if (it.deletedAt == null) items[it.item.id] = it.item
                        else items.remove(it.item.id)
                    }
                    operation.value.variations.forEach { variations[it.id] = it }
                    operation.value.sessions.forEach { sessions[it.id] = it }
                }
        }

    private inline fun ack(write: () -> Unit): PersistenceOutput {
        write()
        return PersistenceOutput.Ack
    }
}
