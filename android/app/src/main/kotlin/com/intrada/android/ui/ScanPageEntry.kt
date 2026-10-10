package com.intrada.android.ui

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.IntentSenderRequest
import androidx.activity.result.PickVisualMediaRequest
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import com.google.android.gms.common.ConnectionResult
import com.google.android.gms.common.GoogleApiAvailability
import com.google.mlkit.vision.documentscanner.GmsDocumentScannerOptions
import com.google.mlkit.vision.documentscanner.GmsDocumentScanning
import com.google.mlkit.vision.documentscanner.GmsDocumentScanningResult
import com.intrada.android.R
import com.intrada.android.core.PhotoFiles
import com.intrada.android.core.SentryReporter
import com.intrada.android.ui.components.FormErrorBanner
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.components.scaled
import com.intrada.shared.PhotoRecognitionStatus
import com.intrada.shared.PhotoRecognitionView
import java.io.IOException
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

@Composable
fun ScanPageEntry(
    recognition: PhotoRecognitionView,
    onCapture: (String) -> Unit,
    modifier: Modifier = Modifier,
    io: CoroutineDispatcher = Dispatchers.IO,
) {
    val context = LocalContext.current
    val photos = remember(context) { PhotoFiles.of(context) }
    val scope = rememberCoroutineScope()
    val canScan =
        remember(context) {
            GoogleApiAvailability.getInstance().isGooglePlayServicesAvailable(context) ==
                ConnectionResult.SUCCESS
        }
    var failure by rememberSaveable { mutableStateOf<String?>(null) }

    fun keep(source: Uri?) {
        source ?: return
        scope.launch {
            val photoId =
                withContext(io) {
                    try {
                        photos.save(context.contentResolver, source)
                    } catch (e: IOException) {
                        SentryReporter.report(e, "photo write")
                        null
                    } catch (e: SecurityException) {
                        SentryReporter.report(e, "photo write")
                        null
                    } catch (e: IllegalArgumentException) {
                        SentryReporter.report(e, "photo write")
                        null
                    }
                }
            failure = if (photoId == null) SAVE_FAILED else null
            photoId?.let(onCapture)
        }
    }

    val picker =
        rememberLauncherForActivityResult(ActivityResultContracts.PickVisualMedia()) { keep(it) }
    val scanner =
        rememberLauncherForActivityResult(ActivityResultContracts.StartIntentSenderForResult()) {
            result ->
            if (result.resultCode == Activity.RESULT_OK) {
                keep(
                    GmsDocumentScanningResult.fromActivityResultIntent(result.data)
                        ?.pages
                        ?.firstOrNull()
                        ?.imageUri
                )
            }
        }
    val photoId = recognition.photoId
    val thumbnail by
        produceState<ImageBitmap?>(null, photoId) {
            value = photoId?.let { withContext(io) { photos.thumbnail(it)?.asImageBitmap() } }
        }

    ScanPageCard(
        recognition,
        thumbnail,
        failure,
        onScan = {
            failure = null
            val activity = context.findActivity()
            if (activity == null) {
                failure = SCAN_UNAVAILABLE
            } else {
                GmsDocumentScanning.getClient(scannerOptions)
                    .getStartScanIntent(activity)
                    .addOnSuccessListener {
                        scanner.launch(IntentSenderRequest.Builder(it).build())
                    }
                    .addOnFailureListener {
                        SentryReporter.report(it, "page scanner")
                        failure = SCAN_UNAVAILABLE
                    }
            }
        },
        onChoose = {
            failure = null
            picker.launch(PickVisualMediaRequest(ActivityResultContracts.PickVisualMedia.ImageOnly))
        },
        modifier = modifier,
        canScan = canScan,
    )
}

@Composable
internal fun ScanPageCard(
    recognition: PhotoRecognitionView,
    thumbnail: ImageBitmap?,
    failure: String?,
    onScan: () -> Unit,
    onChoose: () -> Unit,
    modifier: Modifier = Modifier,
    canScan: Boolean = true,
) {
    Column(modifier.fillMaxWidth().cardSurface()) {
        if (failure != null) {
            FormErrorBanner(
                failure,
                Modifier.padding(horizontal = IntradaSpacing.card)
                    .padding(top = IntradaSpacing.cardCompact)
                    .testTag("itemForm.scanError"),
            )
        }
        if (recognition.photoId != null) {
            ScannedRow(thumbnail, outcome(recognition.status, recognition.readNothing))
        } else {
            PromptRow()
        }
        if (recognition.status == PhotoRecognitionStatus.READING) {
            Spacer(Modifier.height(IntradaSpacing.cardCompact))
        } else {
            Row(
                Modifier.padding(horizontal = IntradaSpacing.controlGap),
                horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
            ) {
                if (canScan) TextAction("Scan the page", "itemForm.scan", onScan)
                TextAction("Choose a photo", "itemForm.choosePhoto", onChoose)
            }
        }
    }
}

@Composable
private fun PromptRow() {
    Row(
        Modifier.fillMaxWidth()
            .padding(horizontal = IntradaSpacing.card)
            .padding(top = IntradaSpacing.card)
            .semantics(mergeDescendants = true) {},
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Image(
            painterResource(R.drawable.ic_scan),
            contentDescription = null,
            modifier = Modifier.size(IntradaIconSize.control.scaled()),
            colorFilter = ColorFilter.tint(IntradaColor.accent),
        )
        Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
            BasicText(
                "Scan a page",
                style = IntradaFont.bodyMedium.copy(color = IntradaColor.accent),
            )
            BasicText(
                "Title, composer and tempo, read off the page",
                style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
            )
        }
    }
}

@Composable
private fun ScannedRow(thumbnail: ImageBitmap?, outcome: String) {
    val shape = RoundedCornerShape(IntradaRadius.badge)
    Row(
        Modifier.fillMaxWidth()
            .padding(horizontal = IntradaSpacing.cardCompact)
            .padding(top = IntradaSpacing.cardCompact),
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        val frame =
            Modifier.size(40.dp, 52.dp)
                .clip(shape)
                .background(IntradaColor.hairline)
                .border(1.dp, IntradaColor.hairline, shape)
        if (thumbnail != null) {
            Image(
                thumbnail,
                contentDescription = "The page you scanned",
                modifier = frame,
                contentScale = ContentScale.Crop,
            )
        } else {
            Box(frame)
        }
        BasicText(
            outcome,
            Modifier.weight(1f)
                .semantics { liveRegion = LiveRegionMode.Polite }
                .testTag("itemForm.scanOutcome"),
            style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
        )
    }
}

/** A failed read must not look like one that worked, after the musician waited for it. */
private fun outcome(status: PhotoRecognitionStatus, readNothing: Boolean): String =
    when {
        status == PhotoRecognitionStatus.READING -> "Reading the page"
        status == PhotoRecognitionStatus.FAILED ->
            "Couldn't read that page. Type the fields instead."
        status == PhotoRecognitionStatus.READY && readNothing -> "Nothing to read on that page."
        else -> "Scanned page"
    }

@Composable
internal fun FieldMark(weak: Boolean, modifier: Modifier = Modifier) {
    val spoken =
        if (weak) "Read from the photo, but not clearly. Worth checking" else "Read from the photo"
    Row(
        modifier
            .padding(horizontal = IntradaSpacing.card)
            .padding(bottom = IntradaSpacing.cardCompact)
            .clearAndSetSemantics { contentDescription = spoken },
        horizontalArrangement = Arrangement.spacedBy(5.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Image(
            painterResource(R.drawable.ic_scan),
            contentDescription = null,
            modifier = Modifier.size(IntradaIconSize.caption.scaled()),
            colorFilter =
                ColorFilter.tint(
                    if (weak) IntradaColor.inkFaintIcon else IntradaColor.inkSecondary
                ),
        )
        BasicText(
            "From the photo",
            style = IntradaFont.smallMedium.copy(color = IntradaColor.inkSecondary),
        )
    }
}

private val scannerOptions =
    GmsDocumentScannerOptions.Builder()
        .setGalleryImportAllowed(false)
        .setPageLimit(1)
        .setResultFormats(GmsDocumentScannerOptions.RESULT_FORMAT_JPEG)
        .setScannerMode(GmsDocumentScannerOptions.SCANNER_MODE_FULL)
        .build()

private tailrec fun Context.findActivity(): Activity? =
    when (this) {
        is Activity -> this
        is ContextWrapper -> baseContext.findActivity()
        else -> null
    }

private const val SAVE_FAILED = "Couldn't save the photo. Try again."
private const val SCAN_UNAVAILABLE =
    "Scanning isn't available on this phone. Choose a photo instead."
