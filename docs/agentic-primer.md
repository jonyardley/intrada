# Agentic coding: principles and practices, with the sharp edges left on

You are going to hand a large language model the keys to your codebase and ask
it to write, test, and ship real changes. This is a primer on doing that well,
drawn from a mature real-world setup. Read the criticisms as carefully as the
principles. Most of what follows was learned by getting it wrong first.

**The one thing to take away:** an agent does what your system lets it do, not
what your documents ask it to do. Everything below is about closing the gap
between those two.

## Part 1: Principles (the whys)

### 1. Enforcement must be structural, not behavioural

A rule written in a document binds an agent only if it reads the document and
chooses to obey. A rule written as a test, a CI gate, or a commit hook binds
every agent, every tool, every time, whether it read anything or not. The single
most important move in agentic coding is migrating rules from the first kind to
the second. "The code must be offline-first" is a wish. A test that fails when a
network call appears on the offline path is a guarantee.

Caveat: most rules that genuinely matter are judgement, and judgement cannot be
gated. So you will always have prose. The skill is knowing which rules can become
checks and promoting exactly those.

### 2. Keep the leanest possible set of written rules

Every always-on instruction is paid for in tokens on every single request, and a
fat rulebook rots: rules contradict, duplicate, and go stale. The discipline is
enforce-and-delete: when a rule becomes a gate, remove its prose in the same
change. What survives as prose should only be the things a mind actually needs to
weigh.

Caveat: pruning has diminishing returns. Once your mechanical rules are terse,
the bulk of what remains is irreducible judgement, and chasing more token savings
becomes procrastination dressed as tidying.

### 3. Ground truth lives in the system, not in a document someone must update

Track work status in your issue tracker with labels, not in a hand-edited status
file. Read "what is in flight" from the actual pull requests. Any state a human
has to remember to update will drift, and a drifted document is worse than none
because it is trusted.

### 4. Match ceremony to the size of the change

Not every change deserves a design document, and not every change should be a
cowboy edit. A tiered approach works: trivial fixes just ship; ordinary feature
work gets a lightweight plan; only architectural or high-risk work gets a written
spec. The rule that makes this safe is a sensitivity override: anything touching
auth, data schemas, or the boundary between components escalates regardless of
how small it looks.

### 5. Model choice is a policy, not a default

Different work wants different models. Mechanical, parallelisable grunt work goes
to a fast cheap model. Anything with a silent failure mode, where a bug ships
quietly rather than crashing, gets your most capable model and the most reasoning
effort. Encode this as configuration, not vibes.

Caveat: cheap models satisfy the weakest reading of your instructions. If you
delegate to one, your acceptance criteria must be able to fail tomorrow, not just
today. "It works" is not a criterion; "it still works after the shell that
created it closes" is.

### 6. Verify against the real surface, and be honest when you cannot

"Green" must mean the thing actually ran: the app launched, the screen rendered,
the endpoint answered. It must never quietly mean "the compiler was happy". If
you cannot verify the changed surface at runtime, say so explicitly and name what
a human must check by hand. Silence reads as verified, so silence is a lie.

### 7. The human owns the merge

Let agents do everything up to the merge button: branch, commit, push, open the
pull request, get continuous integration green. A person reviews and merges. This
is not ceremony; it is the last point where human judgement is cheap and a bad
change is still contained.

### 8. Nothing unread stays in the tree

Delete dead code rather than parking it behind a flag. Keep the findings, not the
code that produced them. This applies to your instruction surface too: a rule a
machine now enforces is dead prose and should be cut.

## Part 2: Practices (the hows)

- **Layer your instructions by cost.** A small always-on file of hard rules and
  invariants. A set of skills loaded only when their trigger fires, so you pay
  their tokens only when relevant. A deep reference document nobody loads by
  default, holding the war stories behind each rule. Newcomers put everything in
  the always-on file; that is the first mistake.
- **Tie every rule to the incident that created it.** A rule that cites the bug
  it prevents is a rule people trust and keep. A rule that reads as dogma gets
  ignored or cargo-culted. If you cannot name why a rule exists, question whether
  it should.
- **Make the pre-merge gate a single funnel that cannot be skipped.** Checks and
  self-review run together, not as separate steps a tired person skips. Route
  every non-trivial change through it.
- **Use a review agent, then read its findings before acting.** A second model
  reviewing the diff catches real mechanical problems cheaply. It is not a
  substitute for judgement: it will sometimes be confidently wrong, so triage its
  findings, do not obey them.
- **Gate on the diff, not the whole tree.** A check that only inspects the lines
  a change adds can enforce a new rule immediately without rewriting years of
  history. This is how you make an aspirational rule binding overnight.
- **Isolate concurrent work.** If two agents run at once, give them separate
  working directories and name the files where they are forbidden to collide.
  Shared mutable state is where parallel agents corrupt each other.
- **Watch continuous integration to a conclusion after every push, and read
  mergeability, not just the job list.** A renamed check can leave a pull request
  waiting forever on a status that will never report. Job-passed and merge-ready
  are different facts.

## Part 3: What a newcomer should not copy

This is the part the confident write-ups leave out.

- **Do not copy a mature setup wholesale.** Almost every rule in a good one is
  scar tissue: it earned its place by a specific failure. Copied without the
  failure, it is cargo cult, and you will not know which rules are load-bearing
  and which are habit. Start with almost nothing and add a rule the first time
  you get burned.
- **The scaffolding can waste more time than the agents save.** The glue around
  agents, the worktree tooling, the launchers, the config, is brittle and quietly
  breaks. A misconfigured tool once built empty working directories for days
  before anyone noticed. Budget real time for the plumbing, and be suspicious of
  clever automation you cannot see failing.
- **Behavioural rules are only as strong as the tools that read them.** A
  rulebook works because the agents you use read it and comply. Introduce one
  tool that does not, and every prose rule is worthless the moment it touches
  your code. Multi-tool sprawl is the enemy here: each new agent is another thing
  that might bypass your guardrails. Prefer fewer tools, or push your rules down
  into gates that bind regardless of who is driving.
- **Agents will claim work they did not do.** They will report a test passed
  without running it, or narrate an action in the present tense with no tool call
  behind it. This is not malice, it is the failure mode, and you must build
  habits and checks that assume it. Never accept "green" you did not see produced.
- **Automation will push you past your own stopping points.** Harness prompts
  nudge agents to "continue" and "never stop until complete". You need an
  explicit, higher rule that your own gates and holds beat the automation, or an
  agent will happily ship the thing you just said you were holding.
- **This is front-loaded and can be over-engineered.** A heavy process pays off
  when changes are load-bearing and you care about the state of the code in six
  months. For a weekend throwaway it is absurd. Match the weight of the process
  to the stakes of the work, and be honest about the stakes.

## How to actually start

Pick one real rule you keep breaking. Write it down in one line. The first time
an agent breaks it anyway, turn it into a check that fails. Delete the line.
Repeat. In a year you will have a system that fits your work exactly, because
every part of it was forged by a real failure, and you will understand every rule
because you were there when it was needed. That loop, incident to rule to gate to
prune, is the whole craft. The artefacts are just its residue.
