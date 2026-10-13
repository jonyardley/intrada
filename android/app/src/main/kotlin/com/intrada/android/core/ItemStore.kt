package com.intrada.android.core

import com.intrada.shared.Item
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.PersistenceOutput
import com.intrada.shared.PracticeSession
import com.intrada.shared.RecordKey
import com.intrada.shared.RecordKind
import com.intrada.shared.StoredItem
import com.intrada.shared.StoredPracticeSession
import com.intrada.shared.StoredRecords
import com.intrada.shared.Variation

/** Answers the core's persistence operations: rows in, rows out, no decisions. */
interface ItemStore {
    fun run(operation: PersistenceOperation): PersistenceOutput
}

// Seed mode and tests: the app keeps its notebook in SharedItemStore (#2421). Deleted pieces and
// sessions stay as tombstones, as they do on disk, so a merge sees the delete.
class InMemoryItemStore(items: List<Item> = emptyList()) : ItemStore {
    private val items =
        LinkedHashMap<String, StoredItem>().apply {
            items.forEach { put(it.id, StoredItem(it, null)) }
        }
    private val sessions = LinkedHashMap<String, StoredPracticeSession>()
    private val variations = LinkedHashMap<String, Variation>()

    @Synchronized
    override fun run(operation: PersistenceOperation): PersistenceOutput =
        when (operation) {
            PersistenceOperation.LoadItems ->
                PersistenceOutput.Items(
                    items.values.filter { it.deletedAt == null }.map { it.item }
                )
            PersistenceOperation.LoadSessions ->
                PersistenceOutput.Sessions(
                    sessions.values.filter { it.deletedAt == null }.map { it.session }
                )
            is PersistenceOperation.SaveItem -> ack { save(operation.value) }
            is PersistenceOperation.SaveItems -> ack { operation.value.forEach(::save) }
            is PersistenceOperation.DeleteItem -> ack { delete(operation) }
            is PersistenceOperation.SaveSession -> ack { save(operation.value) }
            is PersistenceOperation.DeleteSession ->
                ack {
                    sessions[operation.id]?.let {
                        sessions[operation.id] =
                            it.copy(
                                updatedAt = operation.deletedAt,
                                deletedAt = operation.deletedAt,
                            )
                    }
                }
            is PersistenceOperation.RestoreSession ->
                ack {
                    sessions[operation.id]?.let {
                        sessions[operation.id] =
                            it.copy(updatedAt = operation.restoredAt, deletedAt = null)
                    }
                }
            PersistenceOperation.LoadVariations ->
                PersistenceOutput.Variations(variations.values.toList())
            is PersistenceOperation.SaveVariations ->
                ack { operation.value.forEach { variations[it.id] = it } }
            is PersistenceOperation.LoadRecords,
            PersistenceOperation.LoadAllRecords,
            is PersistenceOperation.ApplyMerged -> sync(operation)
        }

    private fun sync(operation: PersistenceOperation): PersistenceOutput =
        when (operation) {
            is PersistenceOperation.LoadRecords ->
                records { kind, id -> RecordKey(kind, id) in operation.value }
            is PersistenceOperation.ApplyMerged ->
                ack {
                    operation.value.items.forEach { items[it.item.id] = it }
                    operation.value.variations.forEach { variations[it.id] = it }
                    operation.value.sessions.forEach { sessions[it.session.id] = it }
                }
            else -> records { _, _ -> true }
        }

    private fun delete(operation: PersistenceOperation.DeleteItem) {
        items[operation.id]?.let { items[operation.id] = StoredItem(it.item, operation.deletedAt) }
    }

    private fun save(item: Item) {
        items[item.id] = StoredItem(item, null)
    }

    // A later save keeps the change time and the delete, as the shared store does (#2581).
    private fun save(session: PracticeSession) {
        val kept = sessions[session.id]
        sessions[session.id] =
            StoredPracticeSession(
                session,
                kept?.updatedAt ?: session.completedAt,
                kept?.deletedAt,
            )
    }

    private fun records(wanted: (RecordKind, String) -> Boolean): PersistenceOutput =
        PersistenceOutput.Records(
            StoredRecords(
                items.values.filter { wanted(RecordKind.ITEM, it.item.id) },
                variations.values.filter { wanted(RecordKind.VARIATION, it.id) },
                sessions.values.filter { wanted(RecordKind.SESSION, it.session.id) },
                emptyList(),
            )
        )

    private inline fun ack(write: () -> Unit): PersistenceOutput {
        write()
        return PersistenceOutput.Ack
    }
}
