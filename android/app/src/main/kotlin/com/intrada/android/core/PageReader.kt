package com.intrada.android.core

import android.graphics.Rect
import com.google.mlkit.vision.common.InputImage
import com.google.mlkit.vision.text.TextRecognition
import com.google.mlkit.vision.text.latin.TextRecognizerOptions
import com.intrada.shared.PageReading
import com.intrada.shared.RecognisedLine
import com.intrada.shared.RecognitionOperation
import com.intrada.shared.RecognitionOutput
import kotlinx.coroutines.tasks.await

/** Hands the core lines with geometry; what a line means is `read_fields` in the core. */
fun interface PageReader {
    suspend fun read(operation: RecognitionOperation): RecognitionOutput

    companion object {
        val Unavailable = PageReader { RecognitionOutput.Failed }
    }
}

/** Android has no on-device suggestion pass, so `suggested` stays empty (#2479). */
class MlKitPageReader(private val photos: PhotoFiles) : PageReader {
    private val recogniser by lazy {
        TextRecognition.getClient(TextRecognizerOptions.DEFAULT_OPTIONS)
    }

    override suspend fun read(operation: RecognitionOperation): RecognitionOutput {
        val photoId =
            when (operation) {
                is RecognitionOperation.ReadPage -> operation.photoId
            }
        val bitmap = photos.bitmap(photoId) ?: return RecognitionOutput.Failed
        val text = recogniser.process(InputImage.fromBitmap(bitmap, 0)).await()
        val lines =
            text.textBlocks.flatMap { block ->
                block.lines.mapNotNull { line ->
                    line.boundingBox?.let {
                        recognisedLine(line.text, it, bitmap.width, bitmap.height, line.confidence)
                    }
                }
            }
        return RecognitionOutput.Page(PageReading(lines, null))
    }
}

/** ML Kit boxes are pixels and can overhang the image; the core's contract is 0..1, top-left. */
fun recognisedLine(
    text: String,
    box: Rect,
    imageWidth: Int,
    imageHeight: Int,
    confidence: Float,
): RecognisedLine {
    fun across(pixels: Int) = (pixels.toFloat() / imageWidth).coerceIn(0f, 1f)
    fun down(pixels: Int) = (pixels.toFloat() / imageHeight).coerceIn(0f, 1f)
    val x = across(box.left)
    val y = down(box.top)
    return RecognisedLine(
        text = text,
        x = x,
        y = y,
        width = across(box.right) - x,
        height = down(box.bottom) - y,
        confidence = confidence,
    )
}
