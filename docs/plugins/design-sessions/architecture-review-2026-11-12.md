# Architecture review and correction register

Recorded 2026-11-12. The user requested a detailed walkthrough of all workflows/components/stages, parallel investigations, and local architecture corrections informed by open PRs. Six read-only agents covered workflows/delivery, runtime/permissions, store/distribution, author/app integration, PR/history, and independent model challenges. The main thread reviewed their evidence and owns documentation edits.

Status: design review in progress. Local analysis, agents, read-only retrieval and documentation are authorized. No implementation, builds, tests, commits, pushes, GitHub comments/reviews, issues/PRs, publishing, releases, new repositories or remotes are authorized by this request.

## Evidence and confidence

- Local source is `a6c5fae8b8f6a92180c02e9188cd3559161c174b`. [State reader](../../../crates/ext-store/src/state.rs), [store transitions](../../../crates/ext-store/src/store.rs), [unpacker](../../../crates/ext-store/src/unpack.rs), and [unpack test](../../../crates/ext-store/src/unpack/tests.rs) were read. The code still contains only the two existing platform foundation crates.
- GitHub main was retrieved at `964c71574affa68da560a19d0efc4bb1f0263aee`. It was not checked out. PR descriptions and patches were read at the exact heads below. Their reported tests were not rerun or independently certified.
- The [confirmed post-call choices](post-call-reconciliation.md), [architecture](../architecture.md), [unit plans](../implementation/README.md), [scenario register](scenarios-and-workflows.md), and existing PR #1/#3 archive provide design/history evidence.
- Primary specification checks used [WebAssembly numeric semantics](https://webassembly.github.io/spec/core/exec/numerics.html), [wasmi 2.0 resource limiting](https://docs.rs/wasmi/2.0.0/wasmi/trait.ResourceLimiter.html), and the [TUF specification](https://theupdateframework.github.io/specification/latest/).

Source facts and explicit user choices are direct evidence. Proposed interleavings are static reasoning, not reproduced failures. New data fields, reset protocols, consent mechanisms, or reduced MVP dependencies remain proposals until agreed. This review uses git/GitHub and repository documents for rationale. Chat, observability, exception tracking and analytics were not searched: the task concerns an unimplemented platform design and no relevant incident/data source was supplied. GitHub issues were not expanded into an unrelated backlog survey.

## Corrections and remaining decisions

| ID | Finding and evidence | Disposition |
|---|---|---|
| AR01 | Architecture section 11 kept only the package current release and leases, although per-app pins and pending consent require other releases. P1 section 7 already identified this conflict | **User approved 2026-11-12:** retain all referenced releases, including app selections/pins, pending consent, dependencies and active work. Architecture and P1 wording reconciled |
| AR02 | P1's `follow: default` plus a pending target does not clearly record the old effective release across successive default updates | Proposed: separate desired selection policy, explicit effective release and pending target. Exact schema/supersession behavior remains open |
| AR03 | New `read_version`/`write_version` fields cannot protect state from older binaries that ignore them. Existing `State::read` rejects `schema > 1`; P1-04 already proposes a refusal boundary | Corrected the architecture's unconditional legacy-readability promise. Shipped-writer inventory, deployment approval and final header representation remain open; do not invent a second migration mechanism |
| AR04 | P1-05 said the installed manifest was already retained. `unpack_into` reads it, extracts only `manifest.files`, and returns the parsed struct; its test expects only `theme.json` on disk | Corrected the source claim. P1 must explicitly retain checked manifest bytes/digest and prove reload/tamper handling |
| AR05 | P6 cutover dependencies omit some dispatcher, authenticated attach, packaging, selected importer and revocation predecessors; later lifecycle work follows wiring | Corrected explicit baseline dependencies and distinguished wiring from complete pilot readiness. A reduced L1-only migration is not approved; no safety/parity gate is removed |
| AR06 | P2's consent comment permits `NeedsConsent` only for a human principal, but approved optional-permission prompts occur during plugin GUI actions | Marked the frozen comment inconsistent. Proposed authenticated GUI-action provenance separate from the plugin principal; exact token/lifetime/replay rules remain open |
| AR07 | “Next host call” cannot fully describe import-free transforms or cross-process observation. P3 already checks fuel slices and proposes polling | Marked the architecture promise incomplete. Agree an observation/linearization/commit contract before claiming global revocation timing. Do not treat proposed polling as approval of a weaker requirement |
| AR08 | P2 already has `EffectReport` and S28 already separates irreversible external effects, but dispatch/user-facing post-commit failure and retry behavior are unspecified | Define partial success, cancellation and retry/idempotency. Do not promise atomic rollback of uploads or build a distributed transaction system. Import-free L1 does not need arbitrary guest effects |
| AR09 | P2's `$1` batch test expects dynamic step-2 targets resolved before step 1 executes; captured selection/new/related targets are underspecified | Choose bounded preflight plus stepwise transactional authorization, or specify a real planning model. Never grant off-scope access through batch references or nested internal calls |
| AR10 | Reused guest globals/memory can make repeated requests return different finite results. Output NaN rejection does not prove determinism, including effects of intermediate NaN payloads | Removed the unconditional proof claim in P4. Define pristine request semantics, reset/state rules, buffer ownership/deallocation, exact input context and cross-platform evidence before promising deterministic reuse |
| AR11 | Guest memory/fuel limits do not cover all host buffers, module caches, pending calls, concurrent copies or blocking brokers | Define a complete host resource model and real broker tests. File-token/symlink races, vetted DNS/actual connections, redirects, streaming limits and cancellation require implementation evidence |
| AR12 | Raster v1 has no imports/resource request field or migration export, yet examples use data dependencies and reopen promises `migrate()`. `HostWorking { linear }` does not identify full color context | Bound v1 state/resources explicitly or add typed resource/migration/context contracts. Installed dependencies alone cannot supply guest input; resource digests belong in execution/cache identity |
| AR13 | P1's manifest contribution shape and P4's module descriptors differ while requiring canonical byte equality; macro `Craft.toml` build inputs and per-app contract aggregation are unspecified | Define one authoritative descriptor schema and explicit projections. Generated validation detects disagreement; it does not by itself make independently declared values incapable of drift |
| AR14 | Package-level `source`/`label` cannot fully describe multiple local/listed releases and accepted ownership generations. Literal namespace-owner equality conflicts with stable IDs after transfer | Proposed: package update binding, per-release provenance/evidence, app/grant accepted binding, and a signed transfer record that preserves initial namespace claims. Keep settings ownership/migration explicit |
| AR15 | Bake keeps a disabled original while “any missing plugin” blocks export. Placeholder byte-for-byte wording also exceeds P6's state-only byte preservation | Define active output dependency closure and preservation guarantees. Prove bake, remove original plugin, export baked output, round-trip preserved original, and explain un-bake requirements |
| AR16 | P4 supplies a transform template but no clear L0 package authoring path; P5 consumes an adapter skeleton without an owned generator delivery task | Assign L0 template, adapter generator/guide and diagnostic examples. A checklist proves hooks exist, not app behavior. Separate package, contract, ABI, manifest, state and crate versions |
| AR17 | Process isolation and a restricted AutoLISP API cannot guarantee a malicious unrestricted native program respects the protected core | Qualified native wording without changing review/consent policy. Supervised termination cannot undo prior external changes; process-tree/descendant handling, frame/output bounds and restart policy remain to specify |
| AR18 | App-scoped manager “Remove” invokes global uninstall; offline table says installation/version selection always needs a connection despite local/retained data | Clarified the offline statement. Removal scope and cross-app consequences must be explicit before changing global state |
| AR19 | Whole-unit P3/P4/P5/P6 arrows obscure cross-unit dependencies; P2 broker interfaces do not deliver executable L2 brokers; basic temporal scope remains open | Added a dependency caveat. Assign broker implementations and distinguish foundations, dormant integration and activation. Preserve the existing broad plan until a smaller MVP sequence is agreed |
| AR20 | Architecture still named the launcher's downloader and FigCraft API levels 1–4 despite recorded corrections | Corrected to shared `craft-update` and API levels 1–6. FigCraft's actual `API_VERSION = 6` definition was checked. No application code changed |

The [walkthrough](system-walkthrough.md) explains the person-visible consequences of these boundaries. Open rows are not implementation permission or accepted replacement policy.

## Minimal model to discuss

The following is a proposed representation of existing concepts, not a frozen new schema:

| Record | Fact it must represent |
|---|---|
| Package | Stable ID and authorized update-source binding |
| Release | Immutable ID/version/digest, checked manifest, acquisition origin and authenticated listing evidence |
| App setup | Enabled contributions/settings, desired selection policy, explicit effective release, pending target and accepted identity |
| Grant | App/package authority, accepted source binding and generation |
| Saved feature | Preserved contribution/state/release reference, separate from the compatible release actually used |
| Execution | Exact resolved release/resource set, leases, principal, captured revision, cancellation and budgets |
| Output dependency set | Contributions/resources actually required for this active render or export, separate from archived inactive data |

These distinctions address concrete update, consent and reopen cases. They do not require a daemon, general package solver, new scripting engine, or distributed transaction coordinator for the local L0/L1 slice.

## What the open PRs contribute

| PR, exact reviewed head | Pattern to retain | Boundary still missing |
|---|---|---|
| [#4](https://github.com/storytold/craft-extensions/pull/4), `6654948663235a0e6fb2f9a715f9bf91b285ad25` | Import enables no app; update preserves existing supported-app choices and clears unsupported choices | Package booleans are not contribution grants or pending-consent state. [Archived patch](../sources/architecture-review-2026-11-12/pr-4.json) |
| [#5](https://github.com/storytold/craft-extensions/pull/5), `58914b08a9638dd2d4e890d73ae40e8dff02bd16` | Replacement policy is checked against serialized current state, preventing a file-source takeover | Tier is not durable source authentication; exact authenticated-digest matching and ownership transfer need the richer model. [Archived patch](../sources/architecture-review-2026-11-12/pr-5.json) |
| [#6](https://github.com/storytold/craft-extensions/pull/6), `85b9d5448a25b117ab120980d1cda0b2af6b1942` | A scoped OS-locked guard provides process liveness; PID/time stay diagnostic | Acquisition/cleanup/deletion are not serialized; state snapshots can become stale, visible leases are written before locking, and some unreadable paths count as unleased. [Archived patch](../sources/architecture-review-2026-11-12/pr-6.json) |
| [#8](https://github.com/storytold/craft-extensions/pull/8), `d9765570967a03c731550b425252173089215c34` | Separate read and write compatibility; deny a write before invoking its mutation closure; retain refused bytes | Schema stays 1, so frozen old writers ignore new headers and can erase them. Its tests exercise updated code, not the historical writer. [Archived patch](../sources/architecture-review-2026-11-12/pr-8.json) |

These are useful narrow changes, not replacements for the full platform model. All four were open at retrieval. No merge/review verdict or comment was submitted.

For #6, one code-derived interleaving is: cleanup reads installed version 1; another process installs version 2; cleanup uses its old snapshot and deletes version 2 as unreferenced. Another is: acquisition creates visible lease metadata; cleanup sees it before the lock is held and removes it; the reader later holds a lease whose release was deleted. These require process/interleaving tests and a shared serialization protocol. They were not reproduced here.

For #8, frozen `State::read` checks only `schema`; serde ignores the new headers. Frozen `Store::update` serializes its smaller model. Adding fields under schema 1 cannot change what that binary does. The existing P1 schema-2 refusal-boundary proposal remains the starting point, subject to deployment approval.

## Proof required by the revised architecture

- Two apps remain on distinct effective releases through chained updates and pending consent; cleanup retains every reference and dependency.
- A frozen old writer refuses the incompatible new state without changing bytes. Unknown/corrupt state never becomes an empty writable store.
- Original checked manifest bytes remain available; altered installed files or descriptors prevent execution.
- Lease acquisition, publication, cleanup, uninstall and process death interleave without deleting live or newly committed releases; uncertain liveness retains data.
- A real plugin GUI action may request optional access while remaining a plugin principal; replayed/headless calls cannot prompt or gain human authority.
- Revocation races cover import-free execution, queued work, cross-process observation and final commit/output publication under an agreed timing contract.
- Fresh/reused instances, tile order, parallel schedules, working-color context and intermediate NaNs cannot invalidate the chosen transform contract; allocation ownership is bounded across repeated calls.
- Missing/failed plugins preserve project data, baked output exports independently of its archived original, and partial exports never appear complete.
- A changed publication record/artifact invalidates old approval. TUF trusted versions and known revocations do not regress under concurrent refresh or ordinary setup repair.
- The integrated EffectCraft acceptance run exercises the selected data kind and transform through GUI/headless paths and the retained baseline parity/cutover requirements.

No item above was run. Local evidence is source/document inspection and documentation verification only. Deferred PC04/PC05/PC07, exact MVP examples, outside review before P12 and two-maintainer native review remain unchanged.

## Discussion record

**AR01, confirmed 2026-11-12:** after the two-app/pending-permission example, the user selected "Retain all referenced releases (Recommended)." Cleanup keeps all releases needed by selected/pinned app setups, pending consent, dependencies, or active work. The reason presented was preserving each app's accepted release between executions; the user supplied no separate rationale. This refines T08's earlier current-plus-leases rule. It does not select schema field names or authorize implementation.

**AR10, question presented 2026-11-12:** proposed the same initial guest state for every logical L1 request, with module caching allowed and instance reuse only when reset preserves that behavior. Explained how a remembered tile counter can change finite results and why behavior should precede pooling decisions. This does not require a particular instantiation strategy, claim its performance, or prohibit a separately designed stateful temporal contract later.

**AR10, response 2026-11-12:** the user selected "Discuss stateful transforms first." No fresh-state rule, stateful contract, reset protocol, or MVP scope change is approved. The discussion below separates persistent feature state from guest-instance memory.

### Stateful transforms: distinctions for discussion

There are separate kinds of state:

| Kind | Example | Required behavior to discuss |
|---|---|---|
| Authored/project state | Parameter values, keyframes, a custom curve, or a saved mask | App preserves versioned data and passes the declared values/state to execution. Saved data need not imply a guest process remains alive |
| Scratch memory and caches | Working buffers, decoded tables, intermediate lookup data | Bounded performance detail. Presence, absence, eviction or reuse must not change the result for the same declared inputs |
| Meaningful processing history | Trails, feedback, a simulation or a stateful temporal filter | Previous state is part of the computation, with explicit ordering, reset/seeking, isolation, cancellation and persistence rules |

A stateless raster result can depend on time, parameters and an explicit seed without depending on earlier calls. Conversely, a history-dependent effect can be deterministic when the contract includes the previous state and declares how it advances. The useful distinction is explicit inputs/state versus accidental hidden call history, not whether the module ever allocates or mutates memory.

Proposed explicit form:

```text
output, next_state = transform(input, parameters, previous_state, context)
```

This is a conceptual contract, not a new ABI. A fresh WebAssembly instance could receive explicit previous state and still implement a stateful effect. A reused instance could implement a stateless effect if its surviving memory cannot change results. Instance reuse and user-visible statefulness are therefore separate decisions.

Meaningful history requires defining state ownership per effect/document/job, which logical step advances it, seeking/replay/checkpoints, how tiled work shares one frame state, preview/final isolation, bounded allocation, and whether any state is project data or only reproducible execution cache. Output and next-state publication must have coherent cancellation/failure behavior; reset cannot silently erase an authored state. Package updates also need state compatibility/migration rules.

The current [P4 sketch](../implementation/P4-contracts-sdk.md) reserves `state_schema` and has no state/migration exports in raster v1. [P3](../implementation/P3-host.md) proposes instance reuse within a job. [P6 D2](../implementation/P6-effectcraft.md) recommends stateless time/frame-rate/seed context, with meaningful temporal history in a later contract; that recommendation is not a new user decision.

Recommendation for discussion: make meaningful state explicit and host-managed, keep internal caches semantically invisible, and choose contract-by-contract whether history is supported. Whether history-dependent processing belongs in the first raster contract or a separate temporal contract remains open. Exact MVP examples remain deferred.
