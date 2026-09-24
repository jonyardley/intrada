export const meta = {
  name: 'audit',
  description: 'The whole-app audit from docs/audit.md: six lane groups, mutations, verification, synthesis',
  whenToUse: 'At the end of a phase or every quarter, after `just audit-sweep`, with Jon\'s go on the cost (docs/audit.md)',
  phases: [
    { title: 'Review', detail: 'one agent per lane group, plus the last audit\'s findings' },
    { title: 'Mutate', detail: 'the proposed mutations, one at a time, each reverted' },
    { title: 'Verify', detail: 'one skeptic per group' },
    { title: 'Synthesise', detail: 'the report and the issue drafts' },
  ],
}

const GROUPS = {
  boundary: ['Principles', 'Simplicity'],
  data: ['Data integrity', 'Resilience'],
  tests: ['Tests that bite'],
  screens: ['Design', 'Consistency', 'Accessibility'],
  safety: ['Security and privacy', 'Scale'],
  upkeep: ['Dependencies and toolchain', 'Drift'],
}
const HIGH_EFFORT = ['data', 'safety']

const a = args || {}
for (const key of ['worktree', 'commit', 'date', 'sweep']) {
  if (!a[key]) throw new Error(`audit: args.${key} is required (docs/audit.md, Running a full audit)`)
}
const unknown = (a.groups || []).filter(g => !GROUPS[g])
if (unknown.length) throw new Error(`audit: unknown groups ${unknown.join(', ')}; the groups are ${Object.keys(GROUPS).join(', ')}`)
if (!a.previousEpic !== !a.previousReport) throw new Error('audit: previousEpic and previousReport go together')
const groups = a.groups || Object.keys(GROUPS)
const wt = a.worktree

const GROUND = `You are one reviewer in intrada's whole-app audit, run against commit ${a.commit} in the worktree ${wt}. Read files by absolute path under that worktree, and prefix every shell command with \`cd ${wt} && \`. Change no file there. The rubric is ${wt}/docs/audit.md: read it first, in full. The sweep for this run is ${wt}/${a.sweep}.`

const FINDING = {
  type: 'object',
  properties: {
    id: { type: 'string', description: 'GROUP-n, e.g. DATA-3' },
    lane: { type: 'string' },
    question: { type: 'string', description: 'the lane question it answers, e.g. "Data integrity 1"' },
    title: { type: 'string', description: 'what the musician or developer would notice, plain words' },
    band: { type: 'integer', minimum: 1, maximum: 4 },
    evidence: { type: 'string', description: 'file:line references and what they show' },
    cost: { type: 'string' },
    fix: { type: 'string' },
    tier: { type: 'string' },
    gate: { type: 'string', description: 'the script check that would catch this class for good, or empty' },
    existingIssue: { type: 'integer', description: 'an open issue that already covers it, or 0' },
  },
  required: ['id', 'lane', 'question', 'title', 'band', 'evidence', 'cost', 'fix', 'tier'],
}

const LANE_SCHEMA = {
  type: 'object',
  properties: {
    ratings: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          lane: { type: 'string' },
          rating: { enum: ['red', 'amber', 'green'] },
          why: { type: 'string' },
        },
        required: ['lane', 'rating', 'why'],
      },
    },
    findings: { type: 'array', items: FINDING },
    mutations: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          file: { type: 'string' },
          line: { type: 'integer' },
          deletion: { type: 'string', description: 'the exact line or lines to delete' },
          expected: { type: 'string', description: 'the tests expected to fail, or "none" for a suspected gap' },
          control: { type: 'boolean' },
          suite: { enum: ['core', 'ios'] },
        },
        required: ['file', 'deletion', 'expected', 'control', 'suite'],
      },
    },
    notCovered: { type: 'array', items: { type: 'string' } },
  },
  required: ['ratings', 'findings', 'notCovered'],
}

const FOLLOWUP_SCHEMA = {
  type: 'object',
  properties: {
    rows: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          issue: { type: 'integer' },
          title: { type: 'string' },
          state: { enum: ['fixed', 'open', 'back again'] },
          evidence: { type: 'string' },
        },
        required: ['issue', 'title', 'state', 'evidence'],
      },
    },
  },
  required: ['rows'],
}

const MUTATION_SCHEMA = {
  type: 'object',
  properties: {
    rows: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          mutation: { type: 'string' },
          expected: { type: 'string' },
          result: { enum: ['red', 'green', 'not run'] },
          failing: { type: 'string' },
        },
        required: ['mutation', 'expected', 'result', 'failing'],
      },
    },
    treeClean: { type: 'boolean' },
  },
  required: ['rows', 'treeClean'],
}

const VERDICT_SCHEMA = {
  type: 'object',
  properties: {
    verdicts: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          id: { type: 'string' },
          verdict: { enum: ['confirmed', 'refuted', 'uncertain'] },
          how: { type: 'string', description: 'the range read, grep or run that decided it' },
        },
        required: ['id', 'verdict', 'how'],
      },
    },
  },
  required: ['verdicts'],
}

const SYNTHESIS_SCHEMA = {
  type: 'object',
  properties: {
    reportPath: { type: 'string' },
    issueDrafts: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          findingId: { type: 'string' },
          title: { type: 'string' },
          body: { type: 'string' },
          labels: { type: 'array', items: { type: 'string' } },
          existingIssue: { type: 'integer' },
        },
        required: ['findingId', 'title', 'body', 'labels'],
      },
    },
    decisions: { type: 'array', items: { type: 'string' } },
  },
  required: ['reportPath', 'issueDrafts', 'decisions'],
}

phase('Review')
log(`Auditing ${a.commit} in ${wt}: ${groups.join(', ')}`)

const reviewThunks = groups.map(g => () =>
  agent(
    `${GROUND}

Your group is "${g}", the lanes ${GROUPS[g].join(', ')}. Answer every question those lanes ask, and nothing outside them. Start from the sweep metrics the rubric names for your lanes, then read the code. A finding needs file:line evidence you read yourself. Rate each lane by the rubric's Ratings section. Name in notCovered everything you did not read or could not run.${g === 'tests' ? ' Propose mutations as the rubric asks, with two controls; do not apply them, a later stage runs them.' : ''}${g === 'screens' ? ` Read the snapshot images under ${wt}/ios/IntradaTests/__Snapshots__. Take a seeded and an empty screenshot of each tab, starting from \`just ios-run\` and \`SEED=0 just ios-run\`; if the simulator is unavailable, say so in notCovered.` : ''}`,
    { label: `review:${g}`, phase: 'Review', schema: LANE_SCHEMA, effort: HIGH_EFFORT.includes(g) ? 'high' : undefined },
  ).then(r => (r ? { group: g, ...r } : null)),
)

const followupThunk = () =>
  a.previousEpic
    ? agent(
        `${GROUND}

Re-check the last audit. Its report is ${wt}/${a.previousReport} and its issues are the sub-issues of epic #${a.previousEpic} (\`gh issue view ${a.previousEpic}\` and \`gh api repos/jonyardley/intrada/issues/${a.previousEpic}/sub_issues\`). For each: fixed (closed, and the fix is still in the code at ${a.commit}), open, or back again (closed, but the code no longer has the fix). One line of evidence each.`,
        { label: 'review:last-audit', phase: 'Review', schema: FOLLOWUP_SCHEMA },
      )
    : Promise.resolve({ rows: [] })

const [followup, ...reviews] = await parallel([followupThunk, ...reviewThunks])
const lanes = reviews.filter(Boolean)
const missing = groups.filter(g => !lanes.some(l => l.group === g))
if (missing.length) log(`No result from: ${missing.join(', ')}. The report must say so under Not covered.`)

phase('Mutate')
const proposed = lanes.flatMap(l => l.mutations || [])
const mutations = proposed.length
  ? await agent(
      `${GROUND}

The review has finished, so you may now edit files in the worktree, one mutation at a time. First record \`git status --porcelain\` as the baseline; it may list the sweep file. For each mutation below: delete exactly the named lines, run the suite (core: \`cargo test -p intrada-core\`; ios: \`just ios-test\`), record red or green and the failing test names, then restore the file with \`git checkout -- <file>\` and confirm \`git status --porcelain\` matches the baseline before the next. Never delete or restore any other file. Run every core mutation; run at most three ios ones and mark the rest "not run". Finish by confirming the status matches the baseline; treeClean is that answer.

${JSON.stringify(proposed, null, 2)}`,
      { label: 'mutate', phase: 'Mutate', schema: MUTATION_SCHEMA },
    )
  : { rows: [], treeClean: true }
if (!mutations || !mutations.treeClean) {
  throw new Error(`audit: the worktree may still hold a mutation; restore ${wt} to its baseline and resume, or the verifiers read mutated code`)
}

phase('Verify')
const verified = await parallel(
  lanes.map(l => () =>
    l.findings.length
      ? agent(
          `${GROUND}

Try to refute each finding below from group "${l.group}". Read the cited lines yourself and look for the guard, test or caller the reviewer missed. Confirm only what you can show; when unsure, answer uncertain.${l.group === 'tests' ? `\n\nThe mutation results:\n${JSON.stringify(mutations, null, 2)}` : ''}

${JSON.stringify(l.findings, null, 2)}`,
          { label: `verify:${l.group}`, phase: 'Verify', schema: VERDICT_SCHEMA, effort: 'high' },
        ).then(v => ({ group: l.group, verdicts: v ? v.verdicts : [] }))
      : Promise.resolve({ group: l.group, verdicts: [] }),
  ),
)

const verdictOf = new Map(verified.filter(Boolean).flatMap(v => v.verdicts).map(v => [v.id, v]))
const judged = lanes.map(l => ({
  ...l,
  findings: l.findings.map(f => ({ ...f, verdict: verdictOf.get(f.id) || { verdict: 'uncertain', how: 'no verifier result' } })),
}))
const counts = { confirmed: 0, refuted: 0, uncertain: 0 }
judged.forEach(l => l.findings.forEach(f => counts[f.verdict.verdict]++))
log(`Findings: ${counts.confirmed} confirmed, ${counts.uncertain} uncertain, ${counts.refuted} refuted`)

phase('Synthesise')
const synthesis = await agent(
  `${GROUND}

Write the report to ${wt}/docs/audit-${a.date.slice(0, 7)}.md with the Write tool (if Write is refused, a Bash heredoc), following the rubric's "The report" section exactly, in plain British English with no dashes of any kind. Order findings by band, then likelihood. Refuted findings go under Dropped with the reason; uncertain ones are carried and marked so. Compare the sweep with the newest earlier file in ${wt}/docs/audit-metrics and rate every lane from the evidence below, including the trend against ${a.previousReport || 'nothing (first run)'}. Leave "The walk" as a heading for Jon's notes. Then return one issue draft per carried finding and per gate (headings: What you would notice, Why it matters, What to do, Where it lives; labels: a horizon plus a kind), skipping any finding that names an existing issue except as a re-rank note, and list the decisions for Jon.

Missing groups: ${missing.join(', ') || 'none'}

The last audit's findings:
${JSON.stringify(followup, null, 2)}

The mutations:
${JSON.stringify(mutations, null, 2)}

The lanes, findings and verdicts:
${JSON.stringify(judged, null, 2)}`,
  { label: 'synthesise', phase: 'Synthesise', schema: SYNTHESIS_SCHEMA },
)

return { counts, missing, synthesis }
