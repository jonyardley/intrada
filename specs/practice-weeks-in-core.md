# Practice weeks and Progress headline numbers in the core

> Tier 3 (a new bridge shape on the `ViewModel`). Issue #2046, split out of
> #1940 and #1999. Native iOS only. Ships as two PRs: the core first, the
> screens in the same sitting.

## Problem

Two parts of the app are worked out in Swift rather than the core.

The Practice tab's "This week" strip (the row of days you swipe between weeks)
reads the phone's calendar for each session's own date, while the core counts
every past session with today's offset from GMT. Near a clock change the strip
and the Progress screen can put the same session on different days, and so in
different weeks.

Three numbers on the Progress screen are also worked out in Swift: the
"Overall mastery" dial, the "Mastery up" card and the "+1.2 this month" line
under the dial. The dial averages only the five most recently marked items,
and hash order picks the five when their dates tie. The line averages this
week's changes, not a month's, counts first marks as a change of zero and is
floored at zero, so a falling week reads as a green "+0.0".

## Decision: one clock

The core's single offset (`LocalClock`) wins (Jon, 2026-09-23, on the issue).
The strip, the streak, the weekly summary and the week bars all bucket a
session by the day it fell on at today's offset. Per-date time zones across
summer time are out of scope. The shell sends nothing new: `SetUtcOffset`
already carries the offset.

## The week strip

`ViewModel.practice_weeks: Vec<PracticeWeekView>`, oldest first. It runs from
the week of the earliest session to this week, and is this week alone when
there are no sessions (or none before it), so it is never empty.

```rust
pub struct PracticeWeekView {
    pub days: Vec<PracticeDayView>,  // seven, Monday first
    pub practised_days: usize,       // days in this week with a session
    pub opening_day: usize,          // index into `days`
}

pub struct PracticeDayView {
    pub date: String,            // "2026-05-27": the shell's selection key
    pub weekday_initial: String, // "W"
    pub day_number: u32,         // 27
    pub full_date: String,       // "Wednesday 27 May"
    pub heading: String,         // "Today", "Yesterday" or the full date
    pub is_today: bool,
    pub is_future: bool,
    pub session_ids: Vec<String>, // newest first
}
```

A day lists ids into `ViewModel.sessions` rather than copies: every session
would otherwise cross the bridge twice on every render.

The opening day carries the rule the Swift had. This week opens on today when
today has practice, otherwise on its most recent earlier practice day,
otherwise on today. A past week opens on its most recent practice day,
otherwise on its Sunday.

Weeks start on Monday, as `compute_weekly_summary` and `compute_weekly_minutes`
count them. Sessions after this week are not shown, as before.

The shell keeps only interaction state: which week page is showing and which
day the musician tapped (by `date`). It no longer holds a calendar for the
strip.

## The Progress numbers

Appended to `AnalyticsView`:

- `overall_mastery: f64`: the mean latest mark of every library item that has
  one, as `MasteryDial`'s own comment describes it; 0 when none do.
- `top_mover: Option<ScoreChange>`: the largest rise this week, across all of
  this week's changes rather than the five listed under Recent mastery; ties go
  to the lower item id. `None` when nothing rose.
- `mastery_change: Option<String>`: the line under the dial, "+0.7 this
  week". The mean change of the items marked again this week (first marks have
  no before, so they are left out), across all of them, rounded half up to one
  decimal place. `None` when that mean is not above zero, so the green
  climbing line only shows a real rise.

`score_changes` keeps its five rows; the unlimited list is internal.

## What the screens PR deletes

- `PracticeWeek` (`PracticeSessionView+Date.swift`) and `PracticeWeekTests`,
  whose cases move to Rust tables.
- `overallMastery`, `topMover` and `avgDelta` in `AnalyticsScreen`.
- `AnalyticsView.score_trends`, `compute_score_trends`, `ItemScoreTrend` and
  `ScorePoint`: the dial was their only reader.

## Tests

Test first, as Rust tables over `LocalClock`:

- Every `PracticeWeekTests` case: Monday to Sunday, days bucketed, sessions
  newest first, the opening day in each branch, the span from the earliest
  week, this week alone with no sessions, the count scoped to its week.
- The clock: a 00:30 BST session lands on its local day; Sunday 23:55 and
  Monday 00:05 fall in different weeks.
- Agreement: this week's sessions in the strip match the weekly summary's
  session count.
- Labels: today, yesterday, a future day, the weekday initials.
- The numbers: no marks, a sixth item, a tie, a fall, a first mark, and the
  half-up rounding.
- A bincode round trip of every new type on the FFI wire (#846).

Swift: the Practice snapshots stay identical; the Progress snapshot is
re-recorded for the new line only.

## Out of scope

- Per-date time zone correctness across summer time.
- The session card and session detail date lines, still on the phone's
  calendar (a follow-up issue).
- "Climbing steadily across N pieces" under the change line.
