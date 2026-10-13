package com.intrada.android.ui

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.IntentSenderRequest
import androidx.activity.result.PickVisualMediaRequest
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.platform.LocalContext
import com.google.android.gms.common.ConnectionResult
import com.google.android.gms.common.GoogleApiAvailability
import com.google.mlkit.vision.documentscanner.GmsDocumentScannerOptions
import com.google.mlkit.vision.documentscanner.GmsDocumentScanning
import com.google.mlkit.vision.documentscanner.GmsDocumentScanningResult
import com.intrada.android.core.SentryReporter

internal class PageCapture(val canScan: Boolean, val scan: () -> Unit, val choose: () -> Unit)

@Composable
internal fun rememberPageCapture(onPage: (Uri) -> Unit, onUnavailable: () -> Unit): PageCapture {
    val context = LocalContext.current
    val page by rememberUpdatedState(onPage)
    val unavailable by rememberUpdatedState(onUnavailable)
    val canScan =
        remember(context) {
            GoogleApiAvailability.getInstance().isGooglePlayServicesAvailable(context) ==
                ConnectionResult.SUCCESS
        }
    val picker =
        rememberLauncherForActivityResult(ActivityResultContracts.PickVisualMedia()) {
            it?.let(page)
        }
    val scanner =
        rememberLauncherForActivityResult(ActivityResultContracts.StartIntentSenderForResult()) {
            result ->
            if (result.resultCode == Activity.RESULT_OK) {
                GmsDocumentScanningResult.fromActivityResultIntent(result.data)
                    ?.pages
                    ?.firstOrNull()
                    ?.imageUri
                    ?.let(page)
            }
        }
    return PageCapture(
        canScan,
        scan = {
            val activity = context.findActivity()
            if (activity == null) {
                unavailable()
            } else {
                GmsDocumentScanning.getClient(scannerOptions)
                    .getStartScanIntent(activity)
                    .addOnSuccessListener {
                        scanner.launch(IntentSenderRequest.Builder(it).build())
                    }
                    .addOnFailureListener {
                        SentryReporter.report(it, "page scanner")
                        unavailable()
                    }
            }
        },
        choose = {
            picker.launch(PickVisualMediaRequest(ActivityResultContracts.PickVisualMedia.ImageOnly))
        },
    )
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

internal const val SCAN_UNAVAILABLE =
    "Scanning isn't available on this phone. Choose a photo instead."

internal const val PHOTO_SAVE_FAILED = "Couldn't save the photo. Try again."
