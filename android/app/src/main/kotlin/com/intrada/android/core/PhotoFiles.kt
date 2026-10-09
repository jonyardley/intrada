package com.intrada.android.core

import android.content.ContentResolver
import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.ImageDecoder
import android.net.Uri
import java.io.File
import java.io.IOException
import kotlin.math.max
import kotlin.math.roundToInt

/**
 * Where an item's photo bytes live; the core holds only the id. Nothing here deletes: removing a
 * photo leaves the bytes for the reaping pass (#1442).
 */
class PhotoFiles(private val directory: File) {
    /** Null for anything that is not a ulid, since the id becomes a path component here. */
    fun file(photoId: String): File? =
        if (Ulid.isValid(photoId)) File(directory, "$photoId.jpg") else null

    /**
     * Bytes first, then whoever names them, so an item can never name a file that was never
     * written. Blocks: call it off the main thread.
     */
    fun save(resolver: ContentResolver, source: Uri): String {
        val decoded =
            ImageDecoder.decodeBitmap(ImageDecoder.createSource(resolver, source)) {
                decoder,
                info,
                _ ->
                // Software, so it can be encoded; the decoder also turns the photo upright.
                decoder.allocator = ImageDecoder.ALLOCATOR_SOFTWARE
                val scale = LONGEST_EDGE.toFloat() / max(info.size.width, info.size.height)
                if (scale < 1f) {
                    decoder.setTargetSize(
                        (info.size.width * scale).roundToInt(),
                        (info.size.height * scale).roundToInt(),
                    )
                }
            }
        val photoId = Ulid.generate()
        val destination = checkNotNull(file(photoId)) { "minted an id that is not a ulid" }
        directory.mkdirs()
        val partial = File(directory, "$photoId.partial")
        try {
            val encoded =
                partial.outputStream().use {
                    decoded.compress(Bitmap.CompressFormat.JPEG, QUALITY, it)
                }
            if (!encoded || !partial.renameTo(destination))
                throw IOException("could not keep the photo")
        } finally {
            partial.delete()
        }
        return photoId
    }

    fun bitmap(photoId: String): Bitmap? = decode(photoId, BitmapFactory.Options())

    fun thumbnail(photoId: String): Bitmap? =
        decode(photoId, BitmapFactory.Options().apply { inSampleSize = THUMBNAIL_SAMPLE })

    private fun decode(photoId: String, options: BitmapFactory.Options): Bitmap? =
        file(photoId)?.takeIf { it.exists() }?.let { BitmapFactory.decodeFile(it.path, options) }

    companion object {
        /** A page stays readable zoomed at this size. */
        const val LONGEST_EDGE = 2048
        const val QUALITY = 80
        private const val THUMBNAIL_SAMPLE = 8
        private const val DIRECTORY = "photos"

        fun of(context: Context) = PhotoFiles(File(context.filesDir, DIRECTORY))
    }
}
