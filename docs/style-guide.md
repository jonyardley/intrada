# Style guide

How we write the prose around the code: pull request bodies, commit messages,
issues, docs, specs, plans and code comments. People and agents both write to
it, and review holds everyone to the same standard (#2100).

Agents write most of this prose, and without a standard it drifts into habits
people stop trusting. Keep this guide short: a style guide nobody finishes
governs nothing.

Words on screen follow [`tone-of-voice.md`](tone-of-voice.md) instead. The
shape of a pull request or issue body (its sections and length) is in the
[shipping skill](../.claude/skills/intrada-shipping/SKILL.md); this guide is how
the sentences inside it read. Comment density and kind are in
[`CLAUDE.md`](../CLAUDE.md#code-style).

## Who the reader is

Assume the reader was not there, may be reading on a phone in a week's time,
and has little time. Use the words they use for things, and explain a technical
word the first time it appears. Test: could someone with no context say what
each sentence means?

## The rules

1. **Lead with the answer.** Put the outcome or the recommendation in the first
   sentence, so a reader who stops there still has the point. A bold lead on a
   list item is the same rule in miniature: a plain statement of what happens.
   Test: read the bold words alone. Can the reader say what happens? An "X,
   not Y" contrast is the usual way to fail it.
2. **Use the reader's words.** Name a feature by what a musician sees on screen,
   and explain a technical word the first time it appears: "CI, the checks
   that run on every push". Names we give our own work ("the lane", "Phase R",
   "B1") are ours alone, so say what the work is instead. A word the reader
   already uses is not jargon, and paraphrasing it is drift. Give as much detail
   as the reader needs, one idea to a sentence.
3. **Make every word carry information.** "In order to" is "to". A sentence
   that announces the next one can go, because the reader spends attention on
   every word whether or not it pays them back. Shorter is not the goal: a word
   that earns its place stays.
4. **Match length to the reader's need.** A pull request body is a summary; a
   spec can run long when the length is doing work. The reader decides from
   the first screen whether to keep going, so what they need to act goes
   there.
5. **Use a list for three or more items.** Put each item on its own line. Keep
   one or two points as prose, because a list of two is slower to read than a
   sentence.
6. **Say it straight.** Use the active voice, the concrete noun and the actual
   number. State what is true rather than what sounds safe, because a reader
   who has to decode a sentence stops trusting the next one.
7. **Show where a claim comes from.** Keep what you inferred distinct from what
   you checked, because a confident wrong sentence costs more than a hedged
   right one. Put the source in the sentence: "the full tier passed on the
   last commit", "the snapshot diff shows". Say what you could not check, and
   who checks it by hand, because silence reads as verified.
8. **Comment only on the reason.** The code says what it does. A comment earns
   its place where the reason is not obvious from the code, and cites the issue
   or incident that makes it so.

## House conventions

- **Spelling:** British English. `scripts/check-dashes.sh` fails a fixed list of
  American spellings on added Markdown lines; an API name such as `Color` goes
  in a code span, spelt the way its owner spells it. A quoted title keeps its
  spelling and carries `<!-- docs-check: quoted -->` on its line.
- **Dashes:** commas, colons and full stops instead of em dashes, en dashes or
  double dashes. The same script fails double dashes on added Markdown lines,
  and em and en dashes almost everywhere else; a command flag goes in a code
  span. Pull request bodies, commits and issues are the reviewer's to check.
- **Handles:** issue and pull request numbers are the only stable handles. A
  phase letter or decision number carries the document it resolves in.
- **Dates:** absolute ("2026-09-24"), never "last week" or "recently", because
  these documents outlive the week they were written in.
- **Voice:** peer to peer. The reader is a colleague, so skip cheerleading and
  coaching.
- **The AI:** "the agent" in prose, and the product or model name only where
  the product itself is meant, such as a model rung or a setting.

## Signs of machine writing

Readers stop trusting text that shows these habits. Review works through the
list one item at a time rather than forming an impression.

- **Announcing a point instead of making it.** Openers such as "It is worth
  noting that" delay the point; start with the point.
- **Words chosen for sound.** "Robust", "seamless" and "leverage" tell the
  reader nothing; name the property or the action.
- **Defining a thing by what it is not.** "Evidence, not reassurance" makes the
  reader do the work; say what it is.
- **Hedging the answer away.** Give the answer, then the one condition that
  would change it.
- **Narrating the work instead of the result.** "First I looked at X, then I
  tried Y" is branch history; say what changed and why.
- **Repeating the summary.** End at the last new point.
- **Performing enthusiasm.** Let the result carry the energy.

## Worked examples

Grow this guide with examples rather than rules, and add a rule only when an
example needed one. When review changes a sentence, add the before, the after
and one line on why. The three below illustrate the form; none is a real
change.

**A pull request body**

- **Before:** "This PR aims to introduce a number of important changes in order
  to improve the overall robustness of the library sync, leveraging a new
  approach that should make things more seamless going forward."
- **After:** "A piece edited offline no longer loses its tags when the phone
  reconnects. The newer edit now wins field by field instead of the whole row.
  Not checked: two phones editing the same piece at once, which needs a hand
  test."
- **Why:** the after names what a musician would notice, says what changed,
  and says what nobody has checked yet.

**A name we gave our own work**

- **Before:** "Lands the B1 slice of the cold-signal lane."
- **After:** "Shows how long since you last played a piece on its library row
  (#N)."
- **Why:** "B1" and "lane" resolve nowhere a reader can look; the outcome and
  the issue number do.

**A commit message**

- **Before:** "fix stuff and address review comments"
- **After:** "Keep a piece's tags when it syncs after an offline edit (#N)", then
  a body saying why the old merge dropped them.
- **Why:** the subject is what `git log` shows a year from now; it names the
  outcome and the issue, and the body carries the reason the diff cannot.
