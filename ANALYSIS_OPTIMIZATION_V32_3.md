# v32.3 Candidate Analysis Optimization

## Scope behavior is intentionally unchanged

- POV demo with a uniquely resolved recorder: only that recorder's kills/building destructions can become candidates.
- STV / `all_players`: every player remains eligible.
- In both modes, only events inside confirmed live playable round intervals are eligible for candidate generation.

## Early gates

`player_death` is now rejected before expensive enrichment when it is outside a live round or outside the applicable POV scope. Live-round lookup uses a binary-searchable `LiveRoundIndex` rather than linearly scanning the round list for every event.

Verbose per-rejection logging is no longer enabled by the GUI's normal batch analysis. Aggregate rejection counts are preserved in `analysis_profile.json` and batch benchmark CSV output.

## Indexed analysis

The analyzer now precomputes tick arrays for player/projectile histories so state lookups do not allocate a temporary list of ticks for every binary search.

Projectile entities are indexed by launcher handle. Airshot analysis therefore examines projectile tracks that could belong to the attacker's weapon handles instead of scanning every projectile in the demo for every qualifying kill.

Damage (`player_hurt`) events are indexed by `(attacker, victim)`. Medic deploys are indexed by Medic, target, and tick. Friendly deaths used by sack-recovery logic are indexed by `(round, team)` and tick.

These indexes preserve STV all-player analysis; they reduce the amount of unrelated evidence examined for each player's kill.

## Concurrency

The current Python analyzer still relies primarily on the existing cross-demo Phase 2 process concurrency. Nested Python multiprocessing is intentionally not enabled because Windows process spawning would duplicate the large state timeline and can make memory use worse.

The eventual Rust analyzer should use one global bounded CPU pool so candidate groups from one or several loaded demos share the same CPU budget without oversubscribing the machine.
