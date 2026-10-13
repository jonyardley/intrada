package com.intrada.android.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import com.intrada.android.R
import com.intrada.android.core.PhotoFiles
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.FormErrorBanner
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

@Composable
internal fun PhotoCard(
    itemId: String,
    photoId: String?,
    send: (Event) -> Boolean,
    modifier: Modifier = Modifier,
    io: CoroutineDispatcher = Dispatchers.IO,
) {
    val context = LocalContext.current
    val photos = remember(context) { PhotoFiles.of(context) }
    val scope = rememberCoroutineScope()
    var failure by rememberSaveable { mutableStateOf<String?>(null) }
    val loaded by
        produceState<LoadedPhoto?>(null, photoId) {
            value = photoId?.let {
                LoadedPhoto(it, withContext(io) { photos.preview(it) }?.asImageBitmap())
            }
        }
    val capture =
        rememberPageCapture(
            onPage = { source ->
                scope.launch {
                    val kept = withContext(io) { photos.keep(context.contentResolver, source) }
                    failure = if (kept == null) PHOTO_SAVE_FAILED else null
                    kept?.let { send(Event.Item(ItemEvent.SetPhoto(itemId, it))) }
                }
            },
            onUnavailable = { failure = SCAN_UNAVAILABLE },
        )
    PhotoCardContent(
        photoId,
        loaded?.takeIf { it.photoId == photoId },
        failure,
        PhotoCardActions(
            onChoose = {
                failure = null
                capture.choose()
            },
            onRemove = {
                failure = null
                send(Event.Item(ItemEvent.ClearPhoto(itemId)))
            },
            loadFull = { photoId?.let { withContext(io) { photos.bitmap(it) }?.asImageBitmap() } },
            onScan =
                if (capture.canScan) {
                    {
                        failure = null
                        capture.scan()
                    }
                } else {
                    null
                },
        ),
        modifier,
    )
}

internal class LoadedPhoto(val photoId: String, val image: ImageBitmap?)

internal class PhotoCardActions(
    val onChoose: () -> Unit,
    val onRemove: () -> Unit,
    val loadFull: suspend () -> ImageBitmap? = { null },
    val onScan: (() -> Unit)? = null,
)

@Composable
internal fun PhotoCardContent(
    photoId: String?,
    loaded: LoadedPhoto?,
    failure: String?,
    actions: PhotoCardActions,
    modifier: Modifier = Modifier,
) {
    var confirmingRemove by rememberSaveable { mutableStateOf(false) }
    var viewing by rememberSaveable { mutableStateOf(false) }
    var capturing by rememberSaveable { mutableStateOf(false) }
    val image = loaded?.image
    val onScan = actions.onScan
    Column(modifier.fillMaxWidth().cardSurface()) {
        PhotoHeader(
            photoId,
            onAdd = { if (onScan == null) actions.onChoose() else capturing = true },
            onRemove = { confirmingRemove = true },
        )
        failure?.let {
            FormErrorBanner(
                it,
                Modifier.padding(horizontal = IntradaSpacing.card)
                    .padding(bottom = IntradaSpacing.cardCompact),
            )
        }
        PhotoBody(photoId, loaded) { viewing = true }
    }
    if (viewing && image != null) PhotoViewer(image, actions.loadFull) { viewing = false }
    if (capturing && onScan != null) {
        CaptureMenu(
            if (photoId == null) "Add a photo" else "Change the photo",
            onScan = {
                capturing = false
                onScan()
            },
            onChoose = {
                capturing = false
                actions.onChoose()
            },
            onDismiss = { capturing = false },
        )
    }
    if (confirmingRemove) {
        ConfirmDialog(
            ConfirmCopy(
                "Remove this photo?",
                if (onScan == null) "You'll need to choose it again."
                else "You'll need to scan or choose it again.",
                "Remove",
                "piece.photo.removeConfirm",
            ),
            onConfirm = {
                confirmingRemove = false
                actions.onRemove()
            },
            onDismiss = { confirmingRemove = false },
        )
    }
}

@Composable
private fun PhotoViewer(
    preview: ImageBitmap,
    loadFull: suspend () -> ImageBitmap?,
    onClose: () -> Unit,
) {
    val currentLoad by rememberUpdatedState(loadFull)
    val full by produceState<ImageBitmap?>(null) { value = currentLoad() }
    Dialog(onClose, DialogProperties(usePlatformDefaultWidth = false)) {
        Box(Modifier.fillMaxSize().background(IntradaColor.ink).safeDrawingPadding()) {
            Image(
                full ?: preview,
                contentDescription = "Photo of the page",
                modifier = Modifier.fillMaxSize().testTag("piece.photo.full"),
                contentScale = ContentScale.Fit,
            )
            IconAction(
                R.drawable.ic_close,
                "Close",
                onClose,
                Modifier.align(Alignment.TopEnd),
                tint = IntradaColor.onAccent,
            )
        }
    }
}

@Composable
private fun CaptureMenu(
    title: String,
    onScan: () -> Unit,
    onChoose: () -> Unit,
    onDismiss: () -> Unit,
) {
    Dialog(onDismissRequest = onDismiss) {
        Column(Modifier.fillMaxWidth().cardSurface()) {
            FieldLabel(
                title,
                Modifier.padding(
                    horizontal = IntradaSpacing.card,
                    vertical = IntradaSpacing.cardCompact,
                ),
            )
            HairlineDivider()
            CaptureRow("Scan the page", "piece.photo.scan", onScan)
            HairlineDivider()
            CaptureRow("Choose a photo", "piece.photo.library", onChoose)
        }
    }
}

@Composable
private fun CaptureRow(title: String, tag: String, onClick: () -> Unit) {
    Box(
        Modifier.fillMaxWidth()
            .heightIn(min = 48.dp)
            .clickable(role = Role.Button, onClick = onClick)
            .testTag(tag)
            .padding(horizontal = IntradaSpacing.card, vertical = IntradaSpacing.cardCompact),
        contentAlignment = Alignment.CenterStart,
    ) {
        BasicText(title, style = IntradaFont.body.copy(color = IntradaColor.ink))
    }
}

private val PHOTO_HEIGHT = 244.dp

@Composable
private fun PhotoHeader(photoId: String?, onAdd: () -> Unit, onRemove: () -> Unit) {
    Row(
        Modifier.padding(start = IntradaSpacing.card, end = IntradaSpacing.controlGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        FieldLabel("Photo", Modifier.weight(1f))
        if (photoId != null) {
            TextAction(
                "Remove",
                "piece.photo.remove",
                onRemove,
                Modifier.semantics { contentDescription = "Remove the photo" },
            )
        }
        TextAction(
            if (photoId == null) "Add" else "Change",
            "piece.photo.choose",
            onAdd,
            Modifier.semantics {
                contentDescription = if (photoId == null) "Add a photo" else "Change the photo"
            },
        )
    }
}

@Composable
private fun PhotoBody(photoId: String?, loaded: LoadedPhoto?, onOpen: () -> Unit) {
    val image = loaded?.image
    when {
        image != null ->
            Image(
                image,
                contentDescription = null,
                modifier =
                    Modifier.padding(horizontal = IntradaSpacing.card)
                        .padding(bottom = IntradaSpacing.cardCompact)
                        .fillMaxWidth()
                        .height(PHOTO_HEIGHT)
                        .clip(RoundedCornerShape(IntradaRadius.badge))
                        .background(IntradaColor.surfaceSunken)
                        .clickable(onClickLabel = "open it full screen", role = Role.Image) {
                            onOpen()
                        }
                        .semantics { contentDescription = "Photo of the page" }
                        .testTag("piece.photo"),
                contentScale = ContentScale.Fit,
            )
        photoId == null || loaded != null ->
            BasicText(
                if (photoId == null) "Keep the page you practise from."
                else "That photo is missing.",
                Modifier.padding(horizontal = IntradaSpacing.card)
                    .padding(bottom = IntradaSpacing.card)
                    .testTag(if (photoId == null) "piece.photo.none" else "piece.photo.missing"),
                style = IntradaFont.body.copy(color = IntradaColor.inkSecondary),
            )
    }
}
