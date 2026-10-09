package com.intrada.android.core

import kotlin.random.Random

/**
 * A photo's file name has to exist on disk before the core hears about it, so the shell mints this
 * one id; the core still refuses one that is not a ulid (`validate_photo_id`).
 */
object Ulid {
    private const val ALPHABET = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"
    private const val RANDOM_BYTES = 10
    private const val LENGTH = 26
    private const val TIMESTAMP_TOP_SHIFT = 40
    private const val BYTE_BITS = 8
    private const val BYTE_MASK = 0xFF
    private const val SYMBOL_BITS = 5
    private const val SYMBOL_MASK = 0x1F
    private const val PADDING_BITS = 2

    fun generate(
        millisecondsSinceEpoch: Long = System.currentTimeMillis(),
        random: Random = Random.Default,
    ): String {
        val timestamp =
            (TIMESTAMP_TOP_SHIFT downTo 0 step BYTE_BITS).map {
                (millisecondsSinceEpoch ushr it).toByte()
            }
        return encode(timestamp + random.nextBytes(RANDOM_BYTES).toList())
    }

    fun isValid(id: String): Boolean = id.length == LENGTH && id.all { it in ALPHABET }

    // 16 bytes as 26 characters of 5 bits, left-padded to 130 bits, which keeps the first
    // character at or below '7'.
    private fun encode(bytes: List<Byte>): String {
        var accumulator = 0
        var pending = PADDING_BITS
        return buildString(LENGTH) {
            for (byte in bytes) {
                accumulator = (accumulator shl BYTE_BITS) or (byte.toInt() and BYTE_MASK)
                pending += BYTE_BITS
                while (pending >= SYMBOL_BITS) {
                    pending -= SYMBOL_BITS
                    append(ALPHABET[(accumulator ushr pending) and SYMBOL_MASK])
                }
            }
        }
    }
}
