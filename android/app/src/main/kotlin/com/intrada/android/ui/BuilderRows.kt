package com.intrada.android.ui

import com.intrada.shared.BuilderRowRef
import com.intrada.shared.Event
import com.intrada.shared.SessionEvent
import com.intrada.shared.SetlistBlockView
import com.intrada.shared.SetlistEntryView

enum class SegmentPosition {
    SINGLE,
    TOP,
    MIDDLE,
    BOTTOM,
}

sealed interface BuilderRow {
    val id: String
    val ref: BuilderRowRef
    val startsUnit: Boolean
    val position: SegmentPosition

    data class Standalone(val block: SetlistBlockView, val entry: SetlistEntryView) : BuilderRow {
        override val id = entry.id
        override val ref = BuilderRowRef.Entry(entry.id)
        override val startsUnit = true
        override val position = SegmentPosition.SINGLE
    }

    data class Header(
        val block: SetlistBlockView,
        val groupId: String,
        val collapsed: Boolean,
        override val position: SegmentPosition,
    ) : BuilderRow {
        override val id = "header-$groupId"
        override val ref = BuilderRowRef.Header(groupId)
        override val startsUnit = true
    }

    data class Nested(
        val block: SetlistBlockView,
        val entry: SetlistEntryView,
        val localIndex: Int,
        override val position: SegmentPosition,
    ) : BuilderRow {
        override val id = entry.id
        override val ref = BuilderRowRef.Entry(entry.id)
        override val startsUnit = false
    }

    data class AddRelated(val block: SetlistBlockView, val groupId: String) : BuilderRow {
        override val id = "add-$groupId"
        override val ref = BuilderRowRef.AddRelated(groupId)
        override val startsUnit = false
        override val position = SegmentPosition.BOTTOM
    }

    companion object {
        fun rows(
            blocks: List<SetlistBlockView>,
            collapsed: Set<String>,
            editing: Boolean,
        ): List<BuilderRow> = blocks.flatMap { blockRows(it, collapsed, editing) }

        private fun blockRows(
            block: SetlistBlockView,
            collapsed: Set<String>,
            editing: Boolean,
        ): List<BuilderRow> {
            val groupId =
                block.groupId
                    ?: return listOfNotNull(
                        block.entries.firstOrNull()?.let { Standalone(block, it) }
                    )
            val folded = groupId in collapsed
            val children = if (folded) 0 else block.related.size + if (editing) 0 else 1
            val header =
                Header(
                    block,
                    groupId,
                    folded,
                    if (children > 0) SegmentPosition.TOP else SegmentPosition.SINGLE,
                )
            val nested =
                block.related
                    .takeIf { !folded }
                    .orEmpty()
                    .mapIndexed { index, entry ->
                        val last = editing && index == block.related.lastIndex
                        Nested(
                            block,
                            entry,
                            index,
                            if (last) SegmentPosition.BOTTOM else SegmentPosition.MIDDLE,
                        )
                    }
            return listOf(header) +
                nested +
                listOfNotNull(AddRelated(block, groupId).takeIf { !editing && !folded })
        }

        /**
         * The row lifted from [from] and dropped before the row at [destination] in the list as it
         * stood, sent as the core reads a drop: the row and its neighbours once it is lifted out
         * (#2231). Null when the row lands where it started.
         */
        fun drop(rows: List<BuilderRow>, from: Int, destination: Int): Event? {
            val remaining = rows.toMutableList()
            val moved = remaining.getOrNull(from)?.also { remaining.removeAt(from) }
            val slot =
                minOf(if (from < destination) destination - 1 else destination, remaining.size)
            return if (moved == null || slot == from) null
            else
                Event.Session(
                    SessionEvent.MoveRow(
                        moved.ref,
                        before = remaining.getOrNull(slot)?.ref,
                        after = remaining.getOrNull(slot - 1)?.ref,
                    )
                )
        }

        /**
         * Where a row dragged [offset] pixels from [from] lands, as a [drop] destination: past the
         * middle of a neighbour, it takes that neighbour's place.
         */
        fun dropDestination(heights: List<Int>, from: Int, offset: Float): Int {
            var travelled = 0f
            return if (offset >= 0) {
                var next = from + 1
                while (next < heights.size && offset > travelled + heights[next] / 2f) {
                    travelled += heights[next]
                    next += 1
                }
                if (next == from + 1) from else next
            } else {
                var previous = from - 1
                while (previous >= 0 && -offset > travelled + heights[previous] / 2f) {
                    travelled += heights[previous]
                    previous -= 1
                }
                previous + 1
            }
        }
    }
}
