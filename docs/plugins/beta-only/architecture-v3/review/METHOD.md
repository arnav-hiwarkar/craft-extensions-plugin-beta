# Method v3: finishing the plugin architecture with one spec, scripts and measured convergence

Date: 2026-11-18. Scope: the method only, not the architecture. Everything it needs is in this folder:

| Path | What it is |
|---|---|
| `spec/schema.json` | Entity kinds, required fields, reference rules, length caps, banned wording, deliverable budgets |
| `spec/shards.json` | The 10 review areas, their lenses and text budgets, plus the integration shard |
| `scripts/*.py` | Deterministic tools (stdlib Python 3, `jq`): `split`, `anchors`, `merge`, `apply`, `try_patch`, `check`, `trace`, `slice`, `snap`, `estimate`, `render` |
| `test/run.sh`, `test/root/` | Fixture spec and a regression test: the clean spec passes, and injected defects trip C02, C04, C05 and C12 |
| `workflow.js` | The runnable Workflow script (plain JS; syntax-checked and dry-run against mock agents) |

## 1. Diagnosis: why v1/v2 did not converge

| Symptom | Root cause | v3 counter |
|---|---|---|
| Every reviewer read 100k–190k words | Review unit = whole document set | Review unit = one area slice (≤ ~20k tokens), then only its diff |
| Documents grew on every fix (100k → 190k) | Fixes were prose additions; nothing capped size | Fixes are JSONL patches; field caps and area text budgets fail the check (C03) |
| Same fact in ten files, drifting | No single source of truth | One spec; all owner documents are rendered from it or cite its ids, and the doc check rejects unknown ids |
| Reviewers rediscovered similar gaps | No ledger; no stable defect key | Ledger with key `target-id|class`; reviewers see it in their slice; the workflow drops seen keys |
| 81 → 58 → 53 → 67 serious gaps, no end | No closed requirement set; "serious" was taste; no stopping measure | Gaps count as serious only against matrix rows or owner rules; capture-recapture plus the discovery-curve stop rule (§5) |
| A reviewer looped on truncated output | Unbounded structured output | `maxItems: 12`, `maxLength` on every field, and a `more` flag |
| Facts checked by reading | No deterministic checks | 15 spec rules and 6 document rules run in seconds (§3) |

The old process's own numbers show how far it was from done. A geometric fit to 81, 58 and 53 gives q = (53/81)^(1/2) = 0.81. The expected tail is 53·q/(1−q) ≈ 224 more serious gaps, and reaching three or fewer new gaps per round would take ln(3/53)/ln(0.81) ≈ 14 more rounds. Each round re-read a growing document set. Round 1 of the second loop then found 67, so the count was not even monotone. That shows the gap count measured reviewer variance, not remaining defects.

## 2. What the method borrows, and from where

| Practice | Source | Used here as |
|---|---|---|
| Inspections: entry and exit criteria, checklists, defect logs, a bounded checking rate | Fagan 1976 (IBM Systems Journal); Gilb & Graham 1993, *Software Inspection* (yield falls once checkers exceed an optimum rate, on the order of a page per hour) | Rubric in the reviewer prompt; slice size bounded by area budgets; the ledger; the stop rule as exit criterion |
| Perspective-based reading | Basili et al. 1996, *Empirical Software Engineering* 1(2) | Shard lenses, plus a separate integration shard for cross-cutting interfaces |
| Capture-recapture for remaining defects | Eick et al. 1992 (ICSE); Briand, El Emam, Freimut & Laitenberger 2000 (IEEE TSE 26(6)); Petersson, Thelin, Runeson & Wohlin 2004 (JSS 72(2)) | Two independent reviewers on sampled shards; Chapman and Chao estimators. The literature says these estimators underestimate when detection probabilities differ and only 2–3 reviewers are used, so v3 uses the upper bound and a second, independent rule |
| Missing-mass estimate | Good 1953 (Biometrika), "Good–Turing" | p_new = f1/n, reported each round as the probability that the next finding is new |
| Reliability growth | Goel & Okumoto 1979 (exponential, NHPP) | Geometric discovery-curve tail as the second stop rule |
| Requirements traceability | Gotel & Finkelstein 1994 (ICRE) | INC/API/REQ → CAP → OP/CON/PERM/CMP → TST, computed by `trace.py` |
| Design structure matrix | Steward 1981; Eppinger & Browning 2012 (MIT Press) | CMP dependency graph: cycle, layering, app-to-app and workstream checks (C10); workstreams derived from it |
| ATAM and quality-attribute scenarios | Kazman, Klein & Clements 2000 (CMU/SEI-2000-TR-004); Bass, Clements & Kazman, *Software Architecture in Practice* | QAS kind: source, stimulus, response and measure, each tied to tests. "Fast" and "never crash" become measurable |
| Design by contract and properties | Meyer 1992; Claessen & Hughes 2000 (QuickCheck) | CON kind with `props` (checkable invariants) and `budget`; property tests are required |
| ADRs | Nygard 2011, "Documenting Architecture Decisions" | DEC kind: one decision per record, with status, choice, rationale and `affects` |
| Design docs, six-pagers, RFCs | Google design docs (Ubl 2020, industrialempathy.com); Amazon six-page narratives and PR/FAQ (Bryar & Carr, *Working Backwards*, 2021); the Rust RFC final comment period; IETF `idnits` | Short narrative deliverables with hard word budgets; owner decisions listed separately; a closing window (the stop rule); and a spec linter in the role `idnits` plays for drafts |

## 3. The single source of truth

### 3.1 Shape

The spec is JSON Lines in `spec/`, one file per kind, one record per line, sorted by id. JSON Lines was chosen because Python reads it with the standard library. Agents edit it as patches, never in place. A diff by id is exact.

| Kind | Holds | Must reference |
|---|---|---|
| REQ | The owner's requirements (no deferral, no plugin network, fast, never crash, common architecture, incumbent parity, permissions plus disclaimer) | `verified_by` QAS/TST |
| INC | One of the 30 top plugins per incumbent (AE, PS, FG): rank, what it does, evidence | `uses_api` and/or `caps` |
| API | One incumbent SDK surface item (an AEGP suite, PF selector, UXP or batchPlay module, Figma API family) with exact symbols as evidence | `caps`, or `decisions: [DEC-COM-no-plugin-network]` for network-only items |
| CAP | A capability a plugin gets in an app | `via` (OP/CON/PERM/CMP) and `tests` |
| OP | A canonical or adapter op with a signature | `perms`, owning `cmp` |
| PERM | A permission name, risk L0–L4 or `never`, and contexts | `tests`; a `notice` at L3/L4; a decision if `never` |
| CON | A contract (`raster.filter@2`, …) with checkable `props` and a numeric `budget` | `cmp` and `tests` |
| CMP | A crate or module: repository, layer and dependencies | exactly one `ws` |
| DEC | A decision (status `decided` or `open-owner`), its options, choice and rationale | `affects` |
| TST | A test of kind unit, prop, integ, bench, fuzz, chaos, ui, flagship or golden, with a quantitative `measure` | `verifies` |
| QAS | A quality scenario (performance, availability, security, modifiability, usability, compatibility) | `reqs` and `tests` |
| FLG | A flagship plugin | `incumbents`, `caps`, `tests`, `ws` |
| WS | A workstream, with milestone and dependencies | — |
| TERM | A glossary term with banned synonyms | — |

Ids follow `KIND-SCOPE-slug`. The scope is COM, SEC, EC, PC or FC for our records and AE, PS or FG for incumbent rows. **The slug is the natural key**: the crate name, permission name, op name, contract@version, or a legacy id such as `k3`, `ec-t50` or `d-a`. As a result, parallel extractors produce the same id for the same thing without coordinating. Legacy ids go in `alias`, and `merge.py` resolves references through aliases.

Every record has `area` (one of 10 shards), `status` (spec, decided, open-owner, proposed or retired; "deferred" does not exist) and `src` anchors into v2 (`common.md:120-134`).

### 3.2 Converting v2 cheaply, and is a cheap model right?

Yes, with three conditions: scripts do everything mechanical, the cheap model never makes design judgments, and extraction quality is *measured*, not assumed.

1. `split.py` cuts the eight source files at `##` boundaries into 24 chunks of about 9–15k words. Every line carries a `doc:line|` prefix. It skips `architecture-v2.md` and `how-plugins-work.md`, which are derivatives and would only add duplicates.
2. **Sonnet (medium effort)** extracts each chunk into `extract/frag/xNN.jsonl`, using the kind guidance and the natural-key id rule. Anything the source calls deferred becomes a CAP with status `proposed`. It is never copied as deferral wording.
3. `anchors.py` measures coverage: the share of *normative* source lines (must, never, every, only, budgets, table rows) that some record's `src` covers. Below 0.9 the extractor repairs once; below that, **Haiku** fills only the listed lines.
4. `merge.py` unions the records by id. It resolves aliases and reports value conflicts, name collisions, near-duplicate capabilities (title Jaccard ≥ 0.6) and unresolved references.
5. **Opus (high effort)** adjudicates only that report, in six partitions by id regex, and writes patches.

Why not Haiku for extraction: the v2 text is dense with exceptions, cross-references and legacy ids, and a dropped "never" is a design defect. To check whether Haiku 5.5 is good enough, run the extraction stage on two chunks with each model. Prefer Haiku when its coverage is within 0.03 of Sonnet's and its C01/C02 errors per record are no more than 1.5× Sonnet's, then pass `args.extractModel: 'haiku'`. At list prices, extraction is about $3–4 with Sonnet, so the pilot saves little money. Do it only if usage limits matter more than dollars.

## 4. Traceability matrix

There are two levels, so the work is shared. **Plugin → SDK surface → capability.** About 30 plugins per app use perhaps 60–120 distinct SDK features, and each feature is mapped to capabilities once. A plugin row then adds only its residual needs: UI parity, performance class and data sizes.

* **Building rows** (`Incumbents` phase): three Sonnet agents, one per incumbent (high effort, WebSearch allowed), are seeded from what v2 already holds: `effectcraft.md` §2 (the AE surface), the flagship lists, the FigCraft and PhotoCraft benchmark sections, and research/05–06. Each writes 30 INC rows and the API rows, with *evidence that names SDK symbols*. That makes rows verifiable and stops invented plugins.
* **Mapping** (`Map` phase, Sonnet high, in parallel with adjudication): each API row gets the CAP ids that achieve it at the same speed or better. A missing capability is created as `proposed`, and round-1 Opus fixers design it.
* **Gaps by script**: `trace.py` writes `reports/trace.csv`, one row per (incumbent row, capability). The state is COVERED, NO-REALISER, NO-TEST, PROPOSED, DANGLING, UNMAPPED, WRONG-APP, OWNER-DECISION (blocked by an open owner decision) or OWNER-EXCLUDED (network-only SDK items, recorded against DEC-COM-no-plugin-network). REQ rows must have verification. Each area's slice carries its trace gaps, and they are mandatory work for the fixer.
* The matrix is also the **closed world for "serious"**: a gap is serious only if it breaks a matrix row, an owner rule, an interface, safety or a hot-path budget. A reviewer cannot raise the bar by taste, which is the main reason the count can fall.

## 5. Deterministic checks (`check.py`; one exit code, JSON report)

| Rule | Checks |
|---|---|
| C01 | Schema: id pattern, kind matches file, unique ids, required fields, enums, known area |
| C02 | References resolve, point at an allowed kind, and are not retired |
| C03 | Field and list length caps; per-area text budget (spec bloat fails here) |
| C04 | No deferral wording (deferred, later phase, future work, TBD, TODO, out of scope, post-beta, nice to have, …) |
| C05 | No network permission names; no network wording without a reference to DEC-COM-no-plugin-network |
| C06 | Coverage: every CAP has realisers and tests, every OP has permissions and an owner, every CON has props, budget and tests, every TST verifies something, and so on |
| C07 | Every decision is referenced; every open owner decision has ≥ 2 options and a recommendation |
| C08 | Status allowed; `proposed` warns during rounds and is an error in `--final` |
| C09 | Every CAP is justified by an INC, API, REQ, FLG or QAS record (warning: unjustified capability, or a missing matrix row) |
| C10 | DSM: no CMP cycles, no dependency on a higher layer, no app-to-app dependency, one workstream per component, workstream dependencies consistent with component dependencies |
| C11 | Vocabulary: banned synonyms from TERM; backticked identifiers must name a spec entity |
| C12 | Permissions: L3/L4 need a third-party notice; never-grantable needs a decision |
| C13 | Hot-path entities need a budget and a bench test |
| C14 | A component that runs plugin code needs an availability scenario (never crash) |
| C15 | No alias shared by two entities |
| D01–D06 | Deliverables: present, word budget per file and in total, required headings, no deferral wording, every cited id exists, no unfilled placeholders |

`try_patch.py` applies one patch in memory and prints the errors it fixes and introduces, so every agent validates its own patch before the barrier. `apply.py` refuses to apply two shards' edits to the same id. It reports them as conflicts, and a Sonnet agent then merges them.

## 6. Sharded, diff-based reviews

**Shards** are the 10 areas in `shards.json` plus `integration`. `slice.py` builds each reviewer's entire input:

* round 1 (or the round's audit shard): every record of the area, in full;
* later rounds: only records changed or added since the last snapshot, with their same-area neighbours in full, plus an index of unchanged titles;
* in every round: one-hop neighbours from other areas (id, title, statement), the area's failing checks, its trace gaps, and the ledger of gaps already found;
* `integration` sees only interfaces: CMP, CON, PERM, DEC, WS, QAS and TERM fields, plus op names.

Area budgets of 45–60k characters keep a full slice near 15–20k tokens, and a diff slice under 10k. Compare 250k tokens per reviewer before.

**Selection per round.** Round 1 reviews everything, with a second independent reviewer on 5 alternating shards. Later rounds review only shards that changed or still fail checks or trace rows, plus one rotating audit shard reviewed in full. Rounds after the first run a second reviewer on the audit shard and on the two shards with the most accepted gaps last round. Reviewer B reads the same slice in reverse order, which decorrelates the two reviewers without changing the population of defects they look for.

**Roles, models and budgets** (overhead ≈ 15k tokens per agent for the system prompt and tools):

| Role | Model / effort | Reads | Output cap |
|---|---|---|---|
| Script runner | Haiku / low | nothing | exit code + 8k chars |
| Extractor | Sonnet / medium | 1 chunk ≈ 12k | ~60 records |
| Gap filler | Haiku / medium | listed lines only | appended records |
| Incumbent builder | Sonnet / high, web | seed sections ≈ 25k + search | 30 INC + 60–120 API rows |
| Adjudicator | Opus / high | merge report partition + sources | patch |
| Mapper | Sonnet / high | CAP digest + one app's rows ≈ 35k | patch |
| Mechanical fixer | Sonnet / medium | check report | patch (C01/C02/C03/C11/C15 only) |
| Reviewer A/B | Opus / high (owner rule) | 1 slice ≤ 20k (diff ≤ 10k) + ≤ 8 lookups | 12 gaps × 6 fields ≤ 300 chars |
| Designer-fixer | Opus / high | the same slice + gaps | patch; triage lists ≤ 40 |
| Conflict merger | Sonnet / high | conflicting patch lines | patch |
| Writers | Opus / high (architecture, explainer); Sonnet / medium (plan documents) | digest ≈ 35k + ≤ 40 lookups | templates within budget |
| Fresh-eyes reader | Opus / high | the 5 deliverables ≈ 28k | ≤ 10 issues |

The fixer is also the verifier. It triages every reviewer gap (accept serious, accept minor, or reject with a reason), merges duplicates, fixes every check and trace failure in its area, and designs every `proposed` record. Its verdicts, not the reviewers', decide what counts toward the estimate. Rejected keys stay in the ledger, so they are never raised again.

## 7. Stopping rule

For round r, let D_r be the number of *new, fixer-accepted, serious* gaps after duplicates are merged. For each double-reviewed shard s, let n1 and n2 be the accepted serious gaps found by A and by B, and m the number found by both.

* Chapman: N̂_s = (n1+1)(n2+1)/(m+1) − 1. Remaining: R_s = N̂_s − (n1 + n2 − m). Variance: V_s = (n1+1)(n2+1)(n1−m)(n2−m) / ((m+1)²(m+2)).
* Chao lower bound (robust when detectability differs), pooled over double shards: R_chao = f1(f1−1) / (2(f2+1)), where f1 = Σ(n1+n2−2m) and f2 = Σm.
* Scale to every reviewed shard by entity weight: k = W_reviewed / W_double.
* R̂ = k · max(Σ R_s, R_chao). R̂_hi80 = R̂ + 1.2816 · k · √(Σ V_s).
* Good–Turing: p_new = f1 / n_detections, reported each round.
* Discovery curve: q̂ = (D_r / D_1)^(1/(r−1)) and tail_r = D_r · q̂ / (1 − q̂), for r ≥ 2 and q̂ < 1.

**Stop after round r when all of these hold:**

1. **Gates**: `check.py` has 0 errors, `trace.py` has 0 gaps, and no record is `proposed`.
2. **Rate**: D_r ≤ 2.
3. **Remaining**: min(R̂_hi80, tail_r) ≤ 3.

There is a hard cap of 4 rounds (`args.maxRounds`). If the cap is reached first, the run still stops, logs it, and hands the owner the gate failures and the estimate. It never stops silently.

Worked check from the fixture: round 1 had n1 = 6, n2 = 5, m = 3 on one shard and 3/3/2 on another, with D = 14. That gave R̂ = 4.9, R̂_hi80 = 8.3 and p_new = 0.41, so the run continued. Round 2 had D = 1, tail 0.08 and R̂_hi80 = 0, with all gates passing, so it stopped. All three parts are computed by `estimate.py` from `ledger/rN.json`, `reports/check.json`, `reports/trace.json` and the spec, and appended to `ledger/rounds.jsonl`.

Known limits:

* Same-model reviewers are correlated, which inflates m and makes the estimate low. The mitigations are the reverse-order reviewer B, the 80% upper bound, the independent discovery-curve rule, the D_r ≤ 2 requirement and the full audit shard each round.
* Shards skipped as unchanged and clean are assumed to have only the residual the previous round estimated.

## 8. Deliverables without bloat

`render.py gen` writes every table from the spec: components (sorted in dependency order), contracts, quality scenarios, matrix summary, workstreams, flagship cards, open and closed decisions, plus the permission and op references. `render.py digest` gives writers one compact list of ids and statements. Writers produce *templates* in `out/src/*.tmpl.md` that insert tables with `{{gen:NAME}}`, and `render.py fill` builds `out/final/`. Facts therefore come from the spec, and prose only explains them.

| Deliverable | Budget (words, tables included) | Written by |
|---|---|---|
| `architecture.md` | 5,000–8,000 | Opus, with the components, contracts, QAS and matrix tables |
| `how-plugins-work.md` | 4,000–6,500 | Opus: one flagship walkthrough per app, plus install, consent, run, update, failure and management |
| `workstreams.md` | 800–2,500 | Sonnet, with the workstreams table and dependency structure from CMP and WS dependencies |
| `flagships.md` | 1,500–4,000 | Generated cards plus one Sonnet intro |
| `decisions.md` | 300–1,800 | Generated: open owner decisions with options and a recommendation first, decided ones as a table |
| **Total** | **15,000–25,000** (D02) | |

The permission vocabulary and the full op catalogue go to `out/final/reference/`. They are outside the reading budget, because they are lookup material. `check.py --docs out/final --final` enforces budgets, required headings, no deferral wording and known ids, and fails on any `proposed` record. A fresh-eyes Opus reader then reports at most 10 owner-comprehension issues. Wording issues are fixed in the templates. Factual issues are fixed as spec patches and re-rendered, never by editing generated text.

The owner-decision list is simply every DEC with status `open-owner`. If the round cap is hit, the run's final log adds the unmet gates and the residual estimate.

## 9. Running it

Prerequisites: everything in this folder, `python3` (standard library only) and `jq`. Nothing touches a git repository. The run can be resumed by phase.

```
Workflow({ scriptPath: "<this folder>/workflow.js", args: { maxRounds: 4, splitWords: 14000, extractModel: "sonnet" } })
// resume after extraction and merge: args.skip = ["extract", "merge", "gate"], args.startRound = 2
// or use the tool's resumeFromRunId with the same script
```

Phases: Extract and Incumbents in parallel → Merge (deterministic merge, then adjudication and mapping in parallel) → Gate (one mechanical pass) → Review rounds (snapshot and slices → reviewers → fixer per shard as a pipeline → apply → conflict merge → checks, trace, estimate → stop decision) → Render (generate → three writers in parallel → fill and doc check → up to 2 fix passes → fresh read → final check). The script returns the rounds run, the final estimate, the doc-check result and the reader issue count.

## 10. Estimates

Model: tokens per agent ≈ 15k overhead + material read + output. This is the same accounting under which the old run's 30 agents came to about 5.6M, an average of about 187k each. Concurrency here is min(16, CPUs − 2) = 8 on this 10-core Mac.

| Stage | Agents | Opus | Sonnet | Haiku | Wall-clock |
|---|---|---|---|---|---|
| Extract (24 chunks, ~8 fills) and incumbents (3) | 36 | — | 1.30M | 0.25M | ~20 min |
| Merge, adjudicate (6) and map (3) | 11 | 0.30M | 0.20M | 0.04M | ~12 min |
| Gate (1 mechanical pass) | 3 | — | 0.05M | 0.04M | ~6 min |
| Round 1 (16 reviews, 11 fixers) | 31 | 1.52M | 0.05M | 0.08M | ~33 min |
| Round 2 (~14 reviews, ~11 fixers, diff slices) | 29 | 1.00M | 0.05M | 0.08M | ~23 min |
| Round 3 (~9 reviews, ~6 fixers) | 19 | 0.57M | — | 0.08M | ~15 min |
| Render (3 writers, doc fix, reader, apply) | 10 | 0.20M | 0.15M | 0.09M | ~30 min |
| **Total (stop after round 3)** | **~139** | **3.6M** | **1.8M** | **0.65M** | **~2 h 20 min** |

A fourth round would add about 0.4M Opus tokens and 12 minutes.

* **Price-weighted tokens.** Opus 5.5 costs $4/$20 per million input/output tokens, Sonnet 5.5 $2/$10 and Haiku 5.5 $0.10/$0.50 (cached list prices dated 2026-11-03). Weighted by price, the total is about 4.5M Opus-equivalent tokens.
* **List-price upper bound.** Assuming no caching and an 85/15 split between input and output tokens, the run costs about $29: Opus $23, Sonnet $6 and Haiku $0.1.
* **Old approach.** One pass took about 5.6M Opus tokens (about $36), 30 agents and 86 minutes, and did not converge. Its own discovery curve projects about 14 more rounds at 1.3M or more each, which grows as the documents grow: roughly 18M or more tokens and 7 or more hours, still with no measure of when it is done.
* **v3.**
  * About 3× fewer tokens to a converged result.
  * About 60% Opus by volume, against 100% before.
  * Per-round cost *falls* from 1.5M to 1.0M to 0.6M, because rounds after the first read only diffs.
  * Each later change, such as the owner answering a decision, costs one diff round: about 0.3–0.6M tokens and about 15 minutes.
* **Where the time goes.** Round 1 fixers are the long pole: they design the `proposed` capabilities the matrix exposes. That is real design work, not overhead. The wall-clock figure assumes about 6 minutes per reviewer and about 10 per round-1 fixer on 8 slots. Shrinking `splitWords` does not help, because extraction is slot-bound.

## 11. Risks and how the method covers them

| Risk | Cover |
|---|---|
| Extraction drops a normative fact | Anchor coverage ≥ 0.9 per chunk; round 1 reviews every record in full; the `src` anchors let a fixer re-read the exact v2 lines |
| Parallel extractors invent different ids for one thing | Natural-key slugs; alias resolution; name-collision and near-duplicate reports; Opus adjudication |
| Fixers bloat the spec | C03 area budgets fail the round; reviewers have a `bloat` class |
| Fixers accept plausible but wrong gaps | Rejection with a reason is part of the fixer's contract; the next round's reviewers see the diff; checks run on every patch |
| A script runner (Haiku) mangles a command or the ledger | Commands are fixed strings; scripts print one JSON line; `estimate.py` fails loudly on bad JSON, and the round then logs a null estimate instead of stopping |
| The estimator underestimates | Upper bound, a second rule, D_r ≤ 2, and a full audit shard each round (§7) |
| The owner rules drift | They are REQ records with tests, and C04/C05/C12/C14 enforce them by script |
