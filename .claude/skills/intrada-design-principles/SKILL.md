---
name: intrada-design-principles
description: "How the app should feel: the 'spend friction deliberately' model (bad friction in admin, setup and navigation removed ruthlessly; good friction kept at the intention and reflection moments), one-primary-action-per-screen, content-over-chrome, progressive disclosure, and reversible-by-default. Carries the visual principles as intent behind Theme.swift's settled tokens (type-colour coding, warmth-biased semantics; the values themselves and their enforcement live in Theme.swift and skill://intrada-design-system) and the dated Open tensions & decisions log, addressable by T-number, that a new UX decision gets appended to rather than decided silently. MUST read before any new surface, layout, flow or interaction, and before recording a new design decision."
---

## Design principles

The interaction and visual principles for how the app should feel, and the
dated Open tensions & decisions log (T1 onward), live in
`docs/design-principles.md`. Read it before deciding a new surface, layout,
flow or interaction. Where a principle pulls against another, or against what
the app does today, that is a decision to make on purpose: settle it and add a
dated entry to the log there, rather than picking a side without a record.

That doc is the *why* and *interaction* layer. The enforcement rules it does
not cover (tokens, primitives, the design hierarchy) are
`skill://intrada-design-system`. What a screen *says*, rather than how it
behaves, is decided against `skill://intrada-tone-of-voice`, the writing layer
built on top of these same principles.
