package com.intrada.android

import com.intrada.android.ui.BuilderRow
import com.intrada.android.ui.SegmentPosition
import com.intrada.shared.BuilderRowRef
import com.intrada.shared.Event
import com.intrada.shared.SessionEvent
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class BuilderRowsTest {
    @Test
    fun aBlockDrawsItsHeaderThenItsExercisesThenTheAddRow() = runTest {
        val setlist = buildingStore().setlist()

        val rows = BuilderRow.rows(setlist.blocks, emptySet(), editing = false)

        assertEquals(
            listOf("Header", "Nested", "AddRelated", "Standalone"),
            rows.map { it::class.simpleName },
        )
        assertEquals(
            listOf(
                SegmentPosition.TOP,
                SegmentPosition.MIDDLE,
                SegmentPosition.BOTTOM,
                SegmentPosition.SINGLE,
            ),
            rows.map { it.position },
        )
    }

    @Test
    fun editingDropsTheAddRowAndClosesTheCardOnTheLastExercise() = runTest {
        val setlist = buildingStore().setlist()

        val rows = BuilderRow.rows(setlist.blocks, emptySet(), editing = true)

        assertEquals(listOf("Header", "Nested", "Standalone"), rows.map { it::class.simpleName })
        assertEquals(SegmentPosition.BOTTOM, rows[1].position)
    }

    @Test
    fun aCollapsedBlockIsItsHeaderAlone() = runTest {
        val setlist = buildingStore().setlist()
        val groupId = setlist.blocks.first().groupId.orEmpty()

        val rows = BuilderRow.rows(setlist.blocks, setOf(groupId), editing = false)

        assertEquals(listOf("Header", "Standalone"), rows.map { it::class.simpleName })
        assertEquals(SegmentPosition.SINGLE, rows.first().position)
    }

    @Test
    fun draggingSatieAboveTheBlockPutsItFirstInTheCore() = runTest {
        val store = buildingStore()
        val rows = BuilderRow.rows(store.setlist().blocks, emptySet(), editing = false)
        val satie = rows.indexOfFirst { it is BuilderRow.Standalone }

        val drop = BuilderRow.drop(rows, satie, 0)
        drop?.let(store::send)

        assertEquals(
            listOf("Gymnopédie No. 1", "Hanon No. 1", "Clair de Lune"),
            store.setlist().titles(),
        )
    }

    @Test
    fun aDropNamesTheNeighboursOnceTheRowIsLiftedOut() = runTest {
        val rows = BuilderRow.rows(buildingStore().setlist().blocks, emptySet(), editing = false)
        val header = rows.first() as BuilderRow.Header
        val satie = rows.last() as BuilderRow.Standalone

        val drop = BuilderRow.drop(rows, 0, rows.size)

        assertEquals(
            Event.Session(
                SessionEvent.MoveRow(
                    BuilderRowRef.Header(header.groupId),
                    before = null,
                    after = BuilderRowRef.Entry(satie.entry.id),
                )
            ),
            drop,
        )
    }

    @Test
    fun aDropWhereTheRowStartedSendsNothing() = runTest {
        val rows = BuilderRow.rows(buildingStore().setlist().blocks, emptySet(), editing = false)
        val last = rows.lastIndex

        assertNull(BuilderRow.drop(rows, last, last))
        assertNull(BuilderRow.drop(rows, last, last + 1))
        assertNull(BuilderRow.drop(rows, rows.size, 0))
    }

    @Test
    fun aDragTakesTheNeighbourPlaceOncePastItsMiddle() {
        val heights = listOf(100, 100, 100, 100)

        assertEquals(0, BuilderRow.dropDestination(heights, 0, 40f))
        assertEquals(2, BuilderRow.dropDestination(heights, 0, 60f))
        assertEquals(3, BuilderRow.dropDestination(heights, 0, 160f))
        assertEquals(4, BuilderRow.dropDestination(heights, 0, 900f))
        assertEquals(3, BuilderRow.dropDestination(heights, 3, -40f))
        assertEquals(1, BuilderRow.dropDestination(heights, 3, -160f))
        assertEquals(0, BuilderRow.dropDestination(heights, 3, -900f))
    }
}
