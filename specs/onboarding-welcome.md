# Onboarding welcome (first-run intro cards)

> Spec for the first-time-user welcome experience.
> Tier 3 per CLAUDE.md (brand-defining first impression — earns a written
> spec even though the implementation surface is web-shell-only).

## Problem

A user who signs into intrada for the first time today lands directly on
the empty Library list ([app.rs:174](../crates/intrada-web/src/app.rs:174)).
There is no welcome, no philosophy, no setting of expectations — they sign
in with Google and are dropped into an empty screen with a `+` button.

This is a missed moment. intrada is not a generic music-practice tool — it
is a specific response to specific frustrations (the "comfort trap",
unstructured practice, the friction of knowing where to start). The first
60 seconds is when a new user decides whether the app understands them.
Right now, the app says nothing at all.

## Goals

1. A new user sees a short, delightful, authentic welcome before reaching
   the empty Library — once per device.
2. The welcome introduces intrada's central insight (the *comfort trap* —
   it's easy to play what you already know; progress lives in the parts
   that take effort) without lecturing or citing research.
3. The welcome speaks in a personal, maker's-note voice — distinct from
   any other practice app — and surfaces the underlying research
   *implicitly* via the worldview the cards convey.
4. The welcome ends with a clear, low-friction first action: add your
   first piece.
5. A user who has seen the welcome does not see it again on the same
   device. A development affordance allows re-triggering it for design
   work and testing.

## Non-goals

- **Personalisation.** No track-selection, no instrument capture, no
  starter-library suggestion. That work belongs to
  [#100 Personalisation](https://github.com/jonyardley/intrada/issues/100).
- **Server-side persistence.** v1 stores the "seen" flag in localStorage
  only. This is intentional — see *Future work* for the upgrade path.
- **A "Show welcome again" affordance in production.** Out of scope for
  v1. Dev-only re-trigger is sufficient for now.
- **Photographic or illustration-heavy treatment.** The voice is
  typographic and intimate; visual production-load belongs to a future
  pass.
- **iOS-platform-specific welcome content.** Same cards on web and iOS;
  iOS adds the existing platform-feel layer (haptics, view transitions,
  safe-area insets) — no separate copy or layout.

## Approach

### Voice and content arc

Five cards: one maker's-note opener followed by a plain-language
walkthrough of the core loop — Capture → Plan → Practice → Track.

| # | Pillar | Copy |
|---|--------|------|
| 1 | Opener | *"Knowing how to practise well is hard. I've struggled with it. So I built this."* |
| 2 | Capture | *"Build a library of pieces and exercises — the things you're actually working on."* |
| 3 | Plan | *"Plan each session with intention. Decide where the effort goes before you pick up the instrument."* |
| 4 | Practice | *"Run focused, timed sessions with real-time reflection — score what happened while it's still fresh."* |
| 5 | Track | *"Track your progress, achieve your goals."* + CTA |

Final-card CTA routes to `/library/new` (the add-piece form). Skip
routes to `/` (the empty Library). Either action records the "seen"
flag.

These lines are the spec — not placeholders. They are derived from a
brainstorming interview and a deliberate content review, chosen because
the opener speaks in a voice no other practice app uses, and the loop
cards introduce exactly the concepts a new user will encounter first.
Edits should be deliberate (PR review, not drift).

### Visual treatment

**Layout.** Pure typographic, full-screen. Each card is the line of
copy, set big, with generous breathing room. Background uses the
existing dark gradient already wired in
[app.rs:102](../crates/intrada-web/src/app.rs:102). One small symbolic
SVG mark per card (e.g., card 2: scattered dots; card 3: those dots
converging to one) for visual rhythm — design in Pencil first.

**Motion.** Spring transitions between cards using the existing Motion
One config (`stiffness: 300, damping: 30`, per CLAUDE.md). Words on the
new card fade-and-rise into place ~200ms after the card lands. On iOS,
wrap card transitions in `document.startViewTransition()` for native
feel.

**Tokens only.** No raw Tailwind colour classes. Type sizes use
`page-title` (or larger) and `text-muted` for any subtitle. Spacing
uses `p-card-comfortable`.

### Flow

- Swipe left to advance, swipe right to go back (mobile);
  tap-anywhere-to-advance (desktop); tap any progress dot to jump.
- Five progress dots at bottom, current one filled.
- "Skip" link top-right — small, low-contrast (`text-muted`), present
  but not pushy. Skips to `/` and records the flag.
- Final card's primary CTA is a Hero-size primary
  [Button](../crates/intrada-web/src/components/button.rs): *"Add your
  first piece →"*. Tapping records the flag and routes to `/library/new`.

**iOS polish** (under `[data-platform="ios"]`):

- `selection` haptic on advance (every card change).
- `success` haptic on the final CTA tap.
- `light` haptic on Skip.
- Safe-area insets on top (Skip link) and bottom (progress dots + CTA).

### Persistence

**localStorage flag.** A single `intrada:welcome-seen` key, written when
the user taps Skip or the final CTA.

```ts
localStorage.setItem('intrada:welcome-seen', '1');
```

The carousel is gated on `localStorage.getItem('intrada:welcome-seen')
=== null`. This check happens once at app boot, after Clerk reports a
signed-in state — see *Architecture* below.

**Trade-off accepted:** a user who signs in on a second device (e.g.,
web first, then iPad) sees the welcome again. For a one-time ~30-second
maker's-note experience, this cost is small. If/when other per-user
preferences land (track, instrument, defaults — see #100), this flag
migrates server-side as part of that work.

### Dev re-trigger affordance

Two paths, both `#[cfg(debug_assertions)]`-gated:

1. **Design catalogue entry.** Add `WelcomeCarousel` to
   [views/design_catalogue.rs](../crates/intrada-web/src/views/design_catalogue.rs)
   so it can be inspected in isolation at `/design`. The design route is
   already debug-gated in `app.rs:316`.
2. **Reset button** in the design catalogue entry that clears
   `localStorage.removeItem('intrada:welcome-seen')` and reloads.

No production "Show welcome again" link. If the need arises later, it
is one line.

### Architecture

This is a **web-shell-only** change. No Crux core changes, no API
changes, no DB migrations.

- **New components** in `crates/intrada-web/src/components/`:
  - `WelcomeCarousel` — the overlay container. Owns card index state,
    swipe/tap handling, progress dots, Skip link, transitions, and the
    localStorage write.
  - `WelcomeCard` — slot for a single card (line of copy + optional
    SVG mark). Used five times by the carousel.
- **Mount point.** Inside
  [`AuthenticatedApp`](../crates/intrada-web/src/app.rs:125), render
  `<WelcomeCarousel />` as a *sibling* of `<main>` (not wrapping it),
  positioned `fixed inset-0 z-10` so it sits visually above the routed
  content. The app underneath continues to mount and fetch data so
  that the empty Library / `/library/new` form is already loaded when
  the carousel dismisses. Visibility is gated on a `RwSignal<bool>`
  set once at `AuthenticatedApp` mount from the localStorage check.
- **No Crux event surface.** Onboarding state in v1 is shell-local UI
  state, not domain state. This intentionally diverges from CLAUDE.md's
  "domain data lives in Crux" rule because the localStorage flag is
  *not* domain data — it is per-device UI state with no server-of-record
  and no cross-shell sharing requirement. When the flag migrates to
  server-side (#100), it becomes domain state and moves into core.
- **No Tauri plugin.** Haptics use the existing `tauri-plugin-haptics`
  bindings. View Transitions use the existing wasm-bindgen bridge.
- **Pencil first.** The five cards and the carousel chrome (dots, Skip,
  CTA) get screens in `design/intrada.pen` before implementation, per
  CLAUDE.md's Pencil workflow.

## Testing

- **Component test (in design catalogue).** Manual, but the catalogue
  entry exercises every state.
- **E2E (Playwright).** New spec covering:
  - Fresh localStorage → sign-in → welcome shows → tap through 5 cards →
    final CTA → land on `/library/new`.
  - Fresh localStorage → sign-in → welcome shows → tap Skip → land on
    `/` → reload → no welcome.
  - localStorage primed → sign-in → no welcome.
- **No core unit tests** — there is no core change.
- **No API integration tests** — there is no API change.

## Risks and unknowns

1. **Tone calibration.** The maker's-note voice is the boldest part of
   the design. If it lands as too personal or self-indulgent in usage
   testing, the cards can be rewritten without architectural change —
   but the architectural commitment to "intimate typographic welcome"
   stays.
2. **Flag-couples-to-device.** If the proportion of users who sign in
   on multiple devices is high, the "shows again on iPad" cost is
   higher than estimated. Acceptable for now; revisit if anecdotal
   feedback says otherwise.
3. **Gradient legibility.** The existing dark gradient was designed
   for the routed app (cards on top of it). At full-screen with no
   card surface, contrast on muted text needs a Pencil pass.

## Future work

- **Server-side persistence** as part of [#100 Personalisation](https://github.com/jonyardley/intrada/issues/100).
  Requires `user_preferences` table + `GET/POST /me/preferences` + a
  new core slice. Migration path: backfill `onboarded_at = now()` for
  any user whose localStorage already had `welcome-seen`.
- **Production "Show welcome again" affordance** (settings or About
  screen). One line once server-side persistence lands.
- **Track-shaped welcome.** Once Musician Tracks (#193) ship, an extra
  card (inserted before card 5, or replacing card 1) can offer a
  light track self-identification. Out of scope for v1 by explicit
  decision earlier in design.
- **Returning users re-onboarding.** A "what's new" treatment is
  conceptually the same component. Different flag, different content.

## References

- [VISION.md](../VISION.md) — five layers, three pillars, core
  principles.
- [docs/research-foundation.md](../docs/research-foundation.md) §3
  Deliberate Practice, §9 Choice Overload — the research the
  "comfort trap" insight rests on.
- [#100 Personalisation epic](https://github.com/jonyardley/intrada/issues/100)
  — where server-side persistence ultimately belongs.
- [#193 Musician tracks](https://github.com/jonyardley/intrada/issues/193)
  — the closed exploration that informs *Future work*.
- [specs/background-audio-plugin.md](background-audio-plugin.md) — Tier-3
  spec template followed here.
