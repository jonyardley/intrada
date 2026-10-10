package com.intrada.android

import com.intrada.android.core.LiveBridge
import com.intrada.shared.Effect
import com.intrada.shared.Event
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.PersistenceOutput
import com.intrada.shared.RecordKey
import com.intrada.shared.RecordKind
import com.intrada.shared.StoredItem
import com.intrada.shared.StoredRecords
import com.intrada.shared.SyncEvent
import com.intrada.shared.SyncOperation
import com.intrada.shared.SyncRecord
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

// Kept apart from BridgeRoundTripTest, which detekt caps for size.
class SyncRoundTripTest {
    // The sync shapes through the real core (#846, specs/icloud-sync.md). Android does not sync
    // yet,
    // but its bindings carry the shapes.
    @Test
    fun anArrivedPieceLoadsTheStoredCopyThenWritesTheMerge() {
        val bridge = LiveBridge()
        val load =
            bridge.update(Event.Sync(SyncEvent.RecordsArrived(listOf(syncRecord(1u))))).single {
                (it.effect as? Effect.Persistence)?.value is PersistenceOperation.LoadRecords
            }
        assertEquals(
            PersistenceOperation.LoadRecords(listOf(RecordKey(RecordKind.ITEM, "piece"))),
            (load.effect as Effect.Persistence).value,
        )

        val written =
            bridge
                .resolve(
                    load.id,
                    PersistenceOutput.Records(
                        StoredRecords(emptyList(), emptyList(), emptyList(), emptyList())
                    ),
                )
                .mapNotNull {
                    ((it.effect as? Effect.Persistence)?.value as? PersistenceOperation.ApplyMerged)
                        ?.value
                }
                .single()
        val items: List<StoredItem> = written.items
        assertEquals(listOf("Etude"), items.map { it.item.title })
        assertEquals(listOf(null), items.map { it.deletedAt })
    }

    @Test
    fun anArrivalForARowThisAppCannotReadIsParkedNotWritten() {
        val bridge = LiveBridge()
        val arrived = syncRecord(1u)
        val load =
            bridge.update(Event.Sync(SyncEvent.RecordsArrived(listOf(arrived)))).single {
                (it.effect as? Effect.Persistence)?.value is PersistenceOperation.LoadRecords
            }
        val answered =
            bridge.resolve(
                load.id,
                PersistenceOutput.Records(
                    StoredRecords(
                        emptyList(),
                        emptyList(),
                        emptyList(),
                        listOf(RecordKey(RecordKind.ITEM, "piece")),
                    )
                ),
            )
        assertEquals(
            listOf(listOf(arrived)),
            answered.mapNotNull {
                ((it.effect as? Effect.Sync)?.value as? SyncOperation.Park)?.value
            },
        )
        assertTrue(
            answered.none {
                (it.effect as? Effect.Persistence)?.value is PersistenceOperation.ApplyMerged
            }
        )
    }

    @Test
    fun aRecordFromANewerAppIsParked() {
        val tooNew = syncRecord(99u)
        val sent =
            LiveBridge().update(Event.Sync(SyncEvent.RecordsArrived(listOf(tooNew)))).mapNotNull {
                (it.effect as? Effect.Sync)?.value
            }
        assertEquals(listOf(SyncOperation.Park(listOf(tooNew)), SyncOperation.Settled), sent)
    }

    private fun syncRecord(schemaVersion: UInt) =
        SyncRecord(
            RecordKind.ITEM,
            "piece",
            schemaVersion,
            "2026-09-21T14:13:20Z",
            null,
            ("{\"id\":\"piece\",\"title\":\"Etude\",\"kind\":\"piece\",\"composer\":null," +
                    "\"key\":null,\"tempo\":null,\"notes\":null,\"tags\":[]," +
                    "\"created_at\":\"2026-09-21T14:13:20Z\",\"updated_at\":\"2026-09-21T14:13:20Z\"}")
                .toByteArray()
                .map { it.toUByte() },
        )
}
