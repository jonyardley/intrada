package com.intrada.android.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.Saver
import androidx.compose.runtime.saveable.mapSaver
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import com.intrada.android.core.Store
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.FormErrorBanner
import com.intrada.android.ui.components.ProfileBadge
import com.intrada.android.ui.components.cardSurface
import com.intrada.shared.Event
import com.intrada.shared.FormErrorTarget
import com.intrada.shared.HighlighterColour
import com.intrada.shared.InstrumentIcon
import com.intrada.shared.Profile
import com.intrada.shared.ProfileEvent
import com.intrada.shared.ProfileField
import com.intrada.shared.ProfileView

class ProfileEditState(
    name: String,
    instrument: String,
    iconChoice: InstrumentIcon?,
    colour: HighlighterColour,
) {
    var name by mutableStateOf(name)
    var instrument by mutableStateOf(instrument)
    var iconChoice by mutableStateOf(iconChoice)
    var colour by mutableStateOf(colour)
    var choosingIcon by mutableStateOf(false)
    var refusal by mutableStateOf<ProfileRefusal?>(null)

    fun profile() = Profile(name, instrument, iconChoice, colour)

    companion object {
        fun of(view: ProfileView) =
            ProfileEditState(
                view.name,
                view.instrument,
                view.icon.takeIf { view.iconChosen },
                view.colour,
            )

        val Saver: Saver<ProfileEditState, Any> =
            mapSaver(
                save = { form ->
                    mapOf(
                        "name" to form.name,
                        "instrument" to form.instrument,
                        "icon" to form.iconChoice?.name,
                        "colour" to form.colour.name,
                        "choosing" to form.choosingIcon,
                        "error" to form.refusal?.message,
                        "field" to form.refusal?.field?.name,
                    )
                },
                restore = ::restored,
            )

        private fun restored(saved: Map<String, Any?>): ProfileEditState? {
            val colour =
                HighlighterColour.entries.firstOrNull { it.name == saved["colour"] } ?: return null
            return ProfileEditState(
                    (saved["name"] as? String).orEmpty(),
                    (saved["instrument"] as? String).orEmpty(),
                    InstrumentIcon.entries.firstOrNull { it.name == saved["icon"] },
                    colour,
                )
                .apply {
                    choosingIcon = saved["choosing"] == true
                    refusal =
                        (saved["error"] as? String)?.let { message ->
                            ProfileRefusal(
                                message,
                                ProfileField.entries.firstOrNull { it.name == saved["field"] },
                            )
                        }
                }
        }
    }
}

class ProfileRefusal(val message: String, val field: ProfileField?)

// The field the core faulted is read before ClearError drops it; nothing closes until the core
// accepts (#1595).
fun Store.saveProfile(profile: Profile): ProfileRefusal? {
    val accepted = sendAccepted(Event.Profile(ProfileEvent.Save(profile)))
    val view = viewModel.value
    val message = view?.error ?: if (accepted) null else PROFILE_SAVE_FAILED
    if (message == null) return null
    val field = (view?.errorTarget as? FormErrorTarget.Profile)?.field
    send(Event.ClearError)
    return ProfileRefusal(message, field)
}

private const val PROFILE_SAVE_FAILED = "Couldn't save your profile. Try again."

@Composable
fun ProfileEditRoute(
    store: Store,
    profile: ProfileView,
    onDone: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val form = rememberSaveable(saver = ProfileEditState.Saver) { ProfileEditState.of(profile) }
    ProfileEditSheet(
        form,
        profile,
        onCancel = onDone,
        onSave = {
            form.refusal = store.saveProfile(form.profile())
            if (form.refusal == null) onDone()
        },
        modifier = modifier,
    )
}

@Composable
fun ProfileEditSheet(
    form: ProfileEditState,
    profile: ProfileView,
    onCancel: () -> Unit,
    onSave: () -> Unit,
    modifier: Modifier = Modifier,
) {
    BackHandler { if (form.choosingIcon) form.choosingIcon = false else onCancel() }
    if (form.choosingIcon) {
        InstrumentIconPicker(
            profile.suggestedIcon,
            form.iconChoice,
            IntradaColor.marker(form.colour),
            onChoose = { form.iconChoice = it },
            onDone = { form.choosingIcon = false },
            modifier = modifier,
        )
        return
    }
    ScreenScaffold(
        "Profile",
        modifier,
        actions = {
            TextAction("Cancel", "profileEdit.cancel", onCancel)
            TextAction("Save", "profileEdit.save", onSave, emphasised = true)
        },
    ) {
        Column(Modifier.fillMaxSize()) {
            form.refusal?.let {
                FormErrorBanner(
                    it.message,
                    Modifier.padding(horizontal = IntradaSpacing.card)
                        .padding(top = IntradaSpacing.cardCompact)
                        .testTag("profileEdit.error"),
                )
            }
            Column(
                Modifier.verticalScroll(rememberScrollState()).padding(IntradaSpacing.card),
                verticalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
            ) {
                IconCard(form, profile.suggestedIcon)
                ProfileNameFields(form, profile.instrumentNames)
                HighlighterSwatches(form.colour, { form.colour = it })
            }
        }
    }
}

@Composable
private fun IconCard(form: ProfileEditState, suggested: InstrumentIcon) {
    val caption =
        when {
            form.iconChoice != null -> "Chosen by you"
            suggested != InstrumentIcon.OTHER -> "Matches ${suggested.tileLabel.lowercase()}"
            else -> "No match"
        }
    Row(
        Modifier.fillMaxWidth()
            .cardSurface()
            .padding(horizontal = IntradaSpacing.card, vertical = IntradaSpacing.cardCompact),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
    ) {
        ProfileBadge(form.iconChoice ?: suggested, IntradaColor.marker(form.colour))
        Column(Modifier.weight(1f)) {
            FieldLabel("Icon")
            BasicText(
                caption,
                style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
            )
        }
        TextAction(
            "Change",
            "profileEdit.changeIcon",
            { form.choosingIcon = true },
            Modifier.semantics { contentDescription = "Change icon" },
        )
    }
}
