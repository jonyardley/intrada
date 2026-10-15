# Android release check

Every Android build is walked against this list before anyone else gets it:
once on the emulator, then once on a real phone, both on the signed release
build. The UI tests and CI's release build catch regressions; this list catches
what they cannot see: how it feels, layout on a real screen, the metronome's
sound, the camera, and closing the app in the middle of a session (#2566).

## Before you start

- **When:** before an APK goes to a tester (#2561) and before a Play release
  (#2499). Walk the emulator first, then the phone. Record each pass as a
  comment on #2566: the build's run number, the device, and the Result column
  filled in.
- **Which build:** the signed APK from the release workflow, never a debug
  build from `just android-run`. CI's own release build is unsigned (see
  `.github/workflows/ci.yml`), so a phone will not install it.
- **Get the build.** Run the release workflow with the Play upload off. Give
  it a few seconds to appear, find its run number, and download its APK when
  the run finishes:

  ```bash
  gh workflow run release-play.yml --ref main -f upload=false
  gh run list --workflow release-play.yml --event workflow_dispatch -L 1
  gh run download <run id> -n intrada-apk -D /tmp/intrada-apk
  ```

- **Install on the emulator.** Boot `intrada-api36` (any `just android-*`
  recipe boots it). A debug build carries a different signature, so remove it
  first or the install fails:

  ```bash
  adb uninstall com.intrada.android
  adb install /tmp/intrada-apk/app-release.apk
  ```

- **Install on a phone.** Turn on USB debugging, plug the phone in, and run the
  same two `adb` commands. Once Play internal testing is live (#2499), install
  from Play instead.
- **Start clean.** The first section needs an empty app. Uninstalling does it;
  so does `adb shell pm clear com.intrada.android`.
- **File faults** as issues in the **1.0.0** milestone, where the Android epic
  (#2220) closes, one fault per issue, with a screenshot and the step number:

  ```bash
  just issue-open "Title" body.md "horizon:now,android,bug" --milestone 1.0.0
  ```

  Put the issue number in the step's Result column. A step passes when every
  "Right" holds and no "Wrong" shows.

## 1. First run

| Step | Do | Look at | Right | Wrong | Result |
| --- | --- | --- | --- | --- | --- |
| 1.1 | Open intrada on a clean install. | The whole screen. | "Intrada", "A notebook for your practice.", three lines for Library, Practice and Progress, a **Set up profile** button and **Skip** at the top. | The Library tab shows instead, or text runs under the status bar. | |
| 1.2 | Tap **Set up profile**. | The profile step. | "Your profile", "All optional, and you can change it later.", Name, Instrument, Highlighter and Icon, and **Save profile**. | Fields clipped at the edges, or the keyboard covers the field you are typing in. | |
| 1.3 | Type a name, pick an instrument and a highlighter colour, tap **Save profile**. | The next step. | "Your first piece" with **Add a piece** and **Add an exercise**. | Nothing happens, or "Couldn't save your profile. Try again." | |
| 1.4 | Tap **Add a piece**, type a title and composer, tap **Add**. | Where you land. | The Practice tab, with your greeting and a **Start here** card whose first step reads "Added" and the piece's title. | The welcome comes back, or the piece is missing from Library. | |
| 1.5 | Close and reopen the app. | The first screen. | The Library tab. The welcome does not come back. | The welcome shows again. | |

## 2. Library

| Step | Do | Look at | Right | Wrong | Result |
| --- | --- | --- | --- | --- | --- |
| 2.1 | Tap the **Library** tab. | The list. | Your piece as a card: title, composer underneath, a coloured bar down the left. | Empty list, or "Pieces and exercises will live here." with a piece saved. | |
| 2.2 | Tap the piece. | The piece page. | Title at the top, **Edit** top right, then Sections, Keys, Variations, Photo, Related exercises and **Delete piece** at the bottom. | A blank page, or "This item is no longer in your library." | |
| 2.3 | Tap **Edit**, change the composer, tap **Save**. | The piece page. | The new composer under the title. | The old composer, or the form will not close. | |
| 2.4 | Under Sections, tap **Add sections** (**Add section** once one exists). In "New section", type "A" for Name and "1 to 8" for Bars, tap **Save**. | The Sections card. | "A" listed with its bars. | Nothing added, or the bars lost. | |
| 2.5 | Under Keys, tap **Choose** (or **Practise in other keys**), pick two keys, tap **Done**. | The Keys card. | Both keys shown. | The sheet will not close, or the keys vanish. | |
| 2.6 | Under Variations, tap **Choose** (or **Choose variations**), type "Hands separately", tap **Add “Hands separately”**, tap **Done**. | The Variations card. | "Hands separately" shown. | It is not saved. | |
| 2.7 | On the Photo card tap **Add**, choose a photo of a page. | The Photo card. | The page appears. Tapping it opens it full screen with **Close**. | "Couldn't save the photo. Try again.", or a blank card. | |

## 3. Add a piece and an exercise

| Step | Do | Look at | Right | Wrong | Result |
| --- | --- | --- | --- | --- | --- |
| 3.1 | On Library, tap **+** at the top right. | The form. | "New Piece", **Cancel** and **Add**, Piece and Exercise pills, a **Scan a page** card ("Title, composer and tempo, read off the page") with **Scan the page** and **Choose a photo**. | No scan card, or **Add** enabled with an empty title. | |
| 3.2 | Tap **Scan the page**. Point it at a printed page and take the scan. | The camera, then the form. | Google's page scanner opens, then "Reading the page", then the fields filled in with "From the photo" under them. | A crash, a black camera, or the form never leaves "Reading the page". | |
| 3.3 | If the scan read nothing useful. | The scan card. | "Nothing to read on that page." or "Couldn't read that page. Type the fields instead.", and the fields still editable. | The form looks as if it worked with nothing filled in. | |
| 3.4 | Tap **Choose a photo** and pick a photo of a page. | The scan card. | Same as 3.2 from "Reading the page". | The picker opens and nothing comes back. | |
| 3.5 | Fix the title if needed, tap **Add**. | Library. | The scanned piece listed. | The form stays open, or the piece is missing. | |
| 3.6 | Tap **+**, tap the **Exercise** pill, title "C major scale", set a tempo marking and beats per minute, tap **Add**. | Library. | The exercise listed. Tapping it opens its edit form, not a piece page. | It saves as a piece. | |
| 3.7 | Open the first piece, under Related exercises tap **Add exercise**, tick "C major scale", tap **Done**. | Related exercises. | "C major scale" listed under the piece. | Not linked. | |

A device without Google Play services cannot scan; there the card says
"Scanning isn't available on this phone. Choose a photo instead." Whether the
scanner runs on the `intrada-api36` emulator has not been tried yet: note what
you see, and check the camera on the phone either way.

## 4. Build a session

| Step | Do | Look at | Right | Wrong | Result |
| --- | --- | --- | --- | --- | --- |
| 4.1 | Tap the **Practice** tab. | The top card. | Either the round play button under "First session" or "Last practised", or an "Up next" card ("Today's plan" when a session length is set) with a "Start · " minutes button, **Change it first** and **Build my own instead**. | Neither shows. | |
| 4.2 | Tap the round play button (or **Build my own instead**). | The builder. | "Build session", **Cancel**, "Add pieces and exercises to build the session.", a Today's length switch and **Add piece or exercise**. | It opens straight into a session. | |
| 4.3 | Tap **Add piece or exercise**, tick the piece, tap **Done**. | The builder list. | A card with the piece's title and "1 related, then piece" under it, "C major scale" beneath marked "Related", and a **Start session** bar at the bottom. | The related exercise is missing, or no start bar. | |
| 4.4 | Tap the three dots on the piece's card, then **Piece settings**. | The settings screen. | The piece's title at the top with **Done**, then Aim, Key, Sections, **Track repetitions** and **Planned duration**. | A blank screen, or "This item is no longer in the session." | |
| 4.5 | Turn on **Track repetitions** and **Planned duration**, set a short duration, tap **Done**. | The builder. | The row shows the planned time, and the start bar reads "Start session · " with the total. | The settings are lost. | |
| 4.6 | Turn on **Today's length** and step it up and down. | The length card. | The minutes change by a step each tap and stop at the ends. | The value jumps or goes negative. | |
| 4.7 | Tap **Cancel**. | The dialog. | "Discard session plan?" with **Discard**. Dismiss it and the plan is still there. | The plan clears without asking. | |

## 5. Focus Player and the metronome

| Step | Do | Look at | Right | Wrong | Result |
| --- | --- | --- | --- | --- | --- |
| 5.1 | Tap **Start session**. | The player. | "Focus · 1 of 2" at the top and "C major scale" (related exercises play before their piece), a timer ring counting up, "Metronome" or the item's tempo, Repetitions with **Not quite** and **Got it**, **Next item**, **Skip this item** and "Next · " the next item. | The tab bar still shows, or the timer does not move. | |
| 5.2 | Leave it for a minute without touching it. | The screen. | It stays on. | The screen dims or locks. | |
| 5.3 | Tap the metronome readout. | Listen, then look under it. | A steady click starts within a moment, the readout shows the beats per minute, **Slower** and **Faster** appear either side, and a bar of beat dots appears underneath with a ring moving along the beat being heard. | No sound, a late or uneven click, or the ring out of step with what you hear. | |
| 5.4 | Tap **Faster** three times, then **Slower** once. Then drag up and down on the readout. | The readout and the sound. | The number and the click change together. | The number changes and the click does not. | |
| 5.5 | Tap the beat dots bar. | The sheet. | "Metronome" with a close button, Time signature pills, "From the piece. Changing it here holds for this session only.", and Sounds on with pattern pills (Every beat, Downbeat and more) and a tap per beat. | The click stops when the sheet opens. | |
| 5.6 | Pick 3/4, choose **Downbeat**, tap the close button. | Listen and look at the bar. | Three dots, only the first filled, and a click on the first beat only. | Four dots, or every beat still clicking. | |
| 5.7 | Turn the phone sideways and back. | Listen. | The click keeps going without a stumble. | It stops or restarts. | |
| 5.8 | Turn the volume down and up with the side buttons. | Listen. | The click follows the media volume. | The buttons change the ringer and the click stays loud. | |
| 5.9 | Tap the readout again. | Listen. | The click stops and **Slower**, **Faster** and the bar go. | It keeps clicking. | |
| 5.10 | Tap **Got it** twice and **Not quite** once, then **Undo**. | Repetitions. | The count follows each tap and **Undo** takes the last one back. | Counts out of step with the taps. | |
| 5.11 | Tap **Trouble spot**, enter First bar 3 and Last bar 4, tap **Add spot**. | The sheet. | It closes with no error. | "Couldn't add the spot. Try again." | |
| 5.12 | Tap the menu at the top right. | The options. | **Skip this item**, **End session early** in red, **Keep practising**. Tap **Keep practising** and nothing changes. | The session ends. | |
| 5.13 | Press the phone's Back. | The screen. | The same options appear; the session is not lost. | The player closes. | |

## 6. Item complete

| Step | Do | Look at | Right | Wrong | Result |
| --- | --- | --- | --- | --- | --- |
| 6.1 | With the metronome running, tap **Next item**. | The screen and the sound. | "Item complete · " with the time, "How did it go?", the item's title, numbered marks, "Tempo reached" with the tempo you finished on, "Reflection · optional", **Save & continue** and **Skip rating**. The click keeps sounding while you mark. | The click cuts out, or the tempo shown is not the one you played. | |
| 6.2 | Press the phone's Back. | The sheet. | It stays. | You lose the sheet or the item. | |
| 6.3 | Tap a mark, type a note, tap **Save & continue**. | The player. | "Focus · 2 of 2" on the piece, the timer back at zero and showing the planned time from 4.5, Repetitions out of the target from 4.5, and the click silent until you start it again. | The first item again, the old item's click still sounding, or "Couldn't save. Try again." | |

## 7. Close the app mid-session and resume

| Step | Do | Look at | Right | Wrong | Result |
| --- | --- | --- | --- | --- | --- |
| 7.1 | With the metronome running, press Home. | Listen. | The click carries on in the background (it stops by itself after ten minutes away). | The click stops at once, or stutters. | |
| 7.2 | Wait just over a minute, then open intrada again. | The player. | The same item, the timer still counting, and a strip reading "Away 1 minute. Leave it out?" with **Leave it out** and a close button. | A fresh session, or the timer reset. | |
| 7.3 | Tap **Leave it out**. | The strip. | The strip goes and stays gone. | The strip stays, or comes back on the next item. | |
| 7.4 | Swipe intrada away from the recent apps screen (on the emulator, `adb shell am force-stop com.intrada.android`). Open it again. | The first screen, then the Practice tab. | Library opens; Practice shows "Pick up where you left off?" with the item count and start time, **Resume** and **Discard**. | No card, or the session is gone. | |
| 7.5 | Tap **Resume**. | The player. | "Focus · 2 of 2", the piece you were on. | Item 1 again, or a crash. | |

## 8. Session complete

| Step | Do | Look at | Right | Wrong | Result |
| --- | --- | --- | --- | --- | --- |
| 8.1 | Carry on from 7.5: tap **Finish session**, then **Save & continue**. | The screen. | "Session complete", the total time, "2 of 2", each item with its mark (the one from 6.3 kept through the restart) and **Add a note**, "A note on the whole session…", Overall marks, **Save session** and **Discard**. | The player again, or items missing. | |
| 8.2 | Press the phone's Back. | The dialog. | "Discard this session?", "This practice won't be saved." Dismiss it. | The session is thrown away. | |
| 8.3 | Change one item's mark, add a note, pick an Overall mark, tap **Save session**. | Where you land. | The tabs again, on Practice, with either "Last practised" over the round play button or an "Up next" card. | The summary will not close. | |
| 8.4 | On Practice, under This week, tap today's session card. | The session page. | "Session", "How it went" with your overall mark, your note, "What you played" with each item. | Marks or notes missing. | |
| 8.5 | Start another session, then from the menu tap **End session early**. | Session complete. | "ended early" next to the count; saving shows "Ended early" on its card. | It reads as a full session. | |
| 8.6 | Start another session, then close the app as in 7.4 and open it again. On Practice tap **Discard**. | The Practice tab. | The card goes, nothing is saved, and it does not come back on the next open. | The card returns, or a session appears in This week. | |

## 9. Progress

| Step | Do | Look at | Right | Wrong | Result |
| --- | --- | --- | --- | --- | --- |
| 9.1 | On a clean install, right after section 1 and before any session, tap **Progress**. | The screen. | "No sessions yet", "Minutes and marks will show here, week by week." and **Build a session**, which opens the builder. | A blank screen or a crash. | |
| 9.2 | After 8.3, tap **Progress**. | The screen. | A line for this week, Overall mastery, "Last five weeks" with a bar for this week, and "Recent mastery" listing what you marked. | Still "No sessions yet", or numbers that do not match your marks. | |
| 9.3 | Tap **Routines**. | The screen. | A "Routines" title on an empty page; nothing is built here yet. | A crash. | |

## 10. Anywhere

| Step | Do | Look at | Right | Wrong | Result |
| --- | --- | --- | --- | --- | --- |
| 10.1 | Turn on dark mode and walk sections 2, 5 and 8 again. | Every screen. | Text readable everywhere. | Dark text on a dark card, or a white flash. | |
| 10.2 | Set the phone's font size to the largest and open Library, the player and Session complete. | Every screen. | Text wraps, nothing overlaps, every button reachable. | Clipped labels or buttons off screen. | |
| 10.3 | On Practice tap the profile button, then **Send feedback**. | The form. | "Feedback" with **Cancel**, **Send** and **Include screenshot**. | It will not open. | |
