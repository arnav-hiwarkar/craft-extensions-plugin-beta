# P2: Operations kit (`craft-ext-ops`)

Status: **build-ready plan, 2026-11-10.** Unit P2 of [the architecture](../architecture.md#7-one-operation-path); deepens [implementation plan section 6, P2](../implementation-plan.md#p2-operations-kit-cratesext-ops). Each PR still needs its own approval.

**Code read.** These files hold the operation paths this crate must serve. Nothing was built or run.

Review qualification, 2026-11-12: [AR06–AR09](../design-sessions/architecture-review-2026-11-12.md) leave consent provenance, revoke/commit boundaries, post-commit effect outcomes and dynamic batch targets unresolved. Affected signature comments are draft contracts. Package grants, plugin identity and headless no-prompt behavior remain approved.

| App | Commit | Files |
|---|---|---|
| EffectCraft | `origin/main` `fb62b9d7` | `crates/engine/src/session/dispatch.rs`, `cmd.rs`, `cmd/batch.rs`, `cmd/app_more.rs`, `cmd/settings.rs`, `history.rs`, `crates/automation/src/confine.rs` |
| FigCraft | `main` `0e86f359` | `crates/engine/src/session/exec.rs`, `undo.rs`, `extensions/plugins.rs` |
| PhotoCraft | `main` `655295b2` | `crates/automation/src/workspace.rs` |

## 1. Goal and exit criteria

**Goal.** One small crate that every app uses to run every caller through the same checks, in the same order. It provides:
- the mandatory invocation, principals, descriptors and markings;
- resolved-argument authorization;
- dispatcher-owned transactions with a post-commit outbox;
- jobs, origin and errors with exit codes;
- the operation inventory format.

The kit carries no app data. It is proven against a **reference app** inside the crate before EffectCraft adopts it (P6).

**Exit criteria (architecture section 22: "denial before side effect; management unreachable by plugins").**

| # | Criterion | Proving test |
|---|---|---|
| X1 | A plugin principal never runs a Management, Core or Internal operation; denial happens before the handler | `reference::plugin_cannot_reach_protected_ops` (handler call counter stays 0) |
| X2 | Every denial happens before any side effect, including queued effects | `reference::denial_leaves_no_trace` (document revision, outbox, journal and events unchanged) |
| X3 | Explicit targets and resolved batch references never escape the captured scope | `authorize::explicit_target_outside_selection_denied`, `authorize::batch_ref_resolved_before_authorization` |
| X4 | Nested edits make one labelled undo step; a failure anywhere rolls back everything and runs no queued effect | `txn::nested_edits_one_step`, `txn::failure_in_step_two_restores_all` |
| X5 | A stale job refuses to commit; queued work keeps its principal; event recursion stops at the cap | `job::stale_job_refuses_commit`, `job::queued_job_keeps_principal`, `events::origin_chain_capped` |
| X6 | An `Invocation` cannot be built from guest bytes or JSON | `invocation::not_deserializable` (`compile_fail` doctest) and `invocation::no_public_field` |
| X7 | Exit codes and error codes are stable | `error::golden_codes` (fixture table) |
| X8 | An unmarked operation, or a breaking change without a major bump, fails the inventory check | `inventory::unmarked_op_fails`, `inventory::breaking_change_needs_major` |

## 2. Code areas the kit must fit (verified)

### EffectCraft `fb62b9d7`, the first adopter

| What | Where | Bearing on the kit |
|---|---|---|
| Caller-less dispatcher | `crates/engine/src/session/dispatch.rs:9` `pub fn execute(&mut self, id: &str, params: Value) -> Result<Value>` | Find spec, then `enabled` (skipped when `layer`/`layers`/`prop`/`keys`/`item`/`items`/`comp` are passed, `:11-20`), `guard::guarded`, `learn_observe`, journal push. **Availability is used as a gate and explicit targets bypass it**: the T04 amendment 1 case |
| Agent entry | `dispatch.rs:35` `execute_checked` adds `commands::check_params` (unknown top-level keys refused; `cmd/schema.rs:107`) | Maps to "control plane JSON validated, unknown fields rejected" |
| Script entry | `dispatch.rs:45` `execute_for_script` increments `script_calls`; `running_for_script()` at `:53` | Becomes a principal (`User(Script)`), not a counter |
| Spec | `crates/engine/src/cmd.rs:105` `CommandSpec { id, label, menu, shortcut, alt_shortcuts, params: &'static str, enabled, run, journal }`; macros `cmd!` (`:123`), `query!` (`:143`); `find` (`:278`) | No kind, exposure or authority field. **About 760 specs** were counted statically from the macros. The comment at `:279` says "~2,000 ids" |
| Hand-written authority lists | `cmd/scripts.rs:34` `RUNS_SCRIPTS`; `cmd/batch.rs:14` `NOT_IN_BATCH`; `crates/automation/src/confine.rs:45` `REFUSED`, `:79` `READS`, `:124` `WRITES`, `:151` `IMPLICIT_WRITES` | All replaced by descriptor data |
| Authority hidden in a generic setter | `cmd/settings.rs:21` `prefs_set`; `:37` refuses when `running_for_script()` and the scripting permissions differ | Becomes a Management operation (P6-E1) |
| Undo | `history.rs:27` whole-project `Arc<Project>` snapshots; `session/edit.rs:10` `edit(label, merge, f)` | Rollback is cheap (an `Arc` swap); merge-key folding for slider drags must survive |
| Existing grouping, three kinds | `cmd/app_more.rs:20` `grouped(s, label, f)`; `cmd/batch.rs:66` `batch` with `fold` (`:123`, atomic by default); `background/jobs.rs:311` `collapse_undo`; script `beginUndoGroup` (`crates/script/src/runtime.rs:248`, `collapse` `:280`) | The kit's transaction protocol unifies all four |
| Panic guard | `system/guard.rs:11` `guarded` uses `catch_unwind` | Stays as the last resort; the kit never relies on it |

### FigCraft and PhotoCraft (the second and third adopters, for fit)

- **FigCraft `0e86f359`.**
  - `crates/engine/src/session/exec.rs:58` `execute_as(agent: Option<&str>, id, params)` and `:71` `execute(id, params)`. One mutation is one undo step, and failure restores a snapshot.
  - `undo.rs:256` `grouped(label, f)` buffers events while `group_depth > 0`. That is already the post-commit event rule.
  - `extensions/plugins.rs:148` `refusal()` is a per-plugin allowlist, and `Access::Read` blocks `history` commands.
- **PhotoCraft `655295b2`.** `crates/automation/src/workspace.rs:327-398` holds per-command `authorize_*` hooks. That is precedent for `requires(resolved)`.

## 3. Module layout and API sketches

```text
crates/ext-ops/src/
  lib.rs          crate root: #![deny(unwrap_used, expect_used, panic, unimplemented, todo, unreachable)]
  principal.rs    Principal, UserSurface, ClientId, AutomationScope, PluginPrincipal            FROZEN
  invocation.rs   Invocation (private fields), entry constructors, child()                       FROZEN
  descriptor.rs   OpId, OpKind, Marking, CallerKind, Exposure, Interaction, OpDescriptor         FROZEN meanings
  authority.rs    EffectiveAuthority::compute(style, grant, host, scope, policy)
  resolve.rs      Resolved, Target, BatchRefs (bounded $N resolution)
  scope.rs        CapturedScope, Selection, Revision, contains()
  authorize.rs    authorize() -> Decision                                                         FROZEN order
  dispatch.rs     dispatch<A: OpApp>(): the one sequence of checks and phases                   FROZEN order, provisional trait
  txn.rs          TxnHost trait, TxnFrame, depth, label with origin
  outbox.rs       Outbox<E>: queue, take_after_commit, drop_on_rollback
  job.rs          JobId, JobState, JobRecord, JobTable, commit_check()
  events.rs       post-commit delivery, origin chain cap
  error.rs        OpError, ErrorCode, exit_code()                                                FROZEN codes
  inventory.rs    operation inventory encoding and breaking-change diff                         FROZEN encoding (shared with P4)
  clock.rs        Clock trait, Deadline (injected; wasm32 has no Instant)
  brokers/        traits only: FileBroker, NetworkBroker, ClipboardBroker; policy types
  reference/      feature "reference": ReferenceApp with every kind of op, used by tests and conformance
```

`craft-ext-ops` depends on `craft-ext-core` only: IDs, `ReleaseKey`, the permission vocabulary and `Eligibility`. It builds for `wasm32-unknown-unknown`, so it spawns no threads and never reads a clock directly.

### Frozen early

```rust
// principal.rs
pub enum Principal {
    User(UserSurface),                                    // the person at the app
    Cli(CliSession),                                      // a CLI the user ran directly: full authority, management included
    Automation { client: ClientId, scope: AutomationScope },   // attached MCP, control channel, attached CLI
    Plugin(PluginPrincipal),
}
pub enum UserSurface { Gui, Script { script: ScriptId } } // app scripts run as the user under the script permission (section 17)
pub enum AutomationScope { Document, DocumentAndManagement { approved: Timestamp } }  // management needs in-app approval, once per session
pub struct PluginPrincipal {
    pub app: AppId, pub release: ReleaseKey, pub contribution: ContributionId,
    pub grant_generation: u64,                            // checked at every host call (P3)
}

// invocation.rs  (no Deserialize, no public fields, no Default)
pub struct Invocation { principal: Principal, origin: OriginChain, scope: CapturedScope,
                        deadline: Option<Deadline>, cancel: CancelToken, txn: Option<TxnId> }
impl Invocation {
    pub fn gui() -> Self;
    pub fn script(id: ScriptId) -> Self;
    pub fn cli(session: CliSession) -> Self;
    pub fn automation(client: ClientId, scope: AutomationScope) -> Self;
    pub fn plugin(authority: PluginAuthority, scope: CapturedScope, deadline: Deadline, cancel: CancelToken) -> Self;
    pub fn child(&self, op: &OpId) -> Result<Self, OpError>;   // same principal; origin += op; depth cap 32 → OpError::TooDeep
    pub fn principal(&self) -> &Principal;  pub fn origin(&self) -> &OriginChain;  pub fn scope(&self) -> &CapturedScope;
}
/// Only from an `Eligibility` that says Available. ext-host builds it; apps never build one by hand.
pub struct PluginAuthority { principal: PluginPrincipal, authority: EffectiveAuthority }
impl PluginAuthority { pub fn from_eligibility(e: &Eligibility, c: &ContributionId, generation: u64) -> Option<Self>; }

// descriptor.rs
pub enum OpKind { Query, Edit, Job, Effect }
pub enum Marking { Normal, Management, Core, Internal }
pub enum CallerKind { Gui, Script, Cli, Automation, Plugin }
pub struct Exposure(u8);                          // bit set of CallerKind
impl Exposure { pub const fn default_for(marking: Marking, cli_or_automation: bool) -> Exposure; }
pub enum Interaction { None, Optional, Required }
pub struct OpDescriptor {
    pub id: OpId, pub kind: OpKind, pub marking: Marking, pub exposure: Exposure, pub interaction: Interaction,
    pub since: OpVersion, pub deprecated: Option<OpVersion>,
    pub args: fn() -> Schema, pub result: fn() -> Schema,     // schemars (MIT)
    pub requires: fn(&Resolved) -> PermissionSet,
}

// authorize.rs  — order is FROZEN
pub enum Decision { Allow, Deny(DenyReason), NeedsConsent(PermissionRequest) }
pub fn authorize(inv: &Invocation, d: &OpDescriptor, r: &Resolved, a: &EffectiveAuthority, p: &MachinePolicy) -> Decision;
// 1 exposure for the caller kind  2 marking (Management needs Cli, Gui or approved Automation; Core/Internal never Plugin)
// 3 requires(resolved) ⊆ authority  4 every target inside the captured scope  5 machine policy
// Consent provenance is unresolved (AR06): a plugin GUI action must retain its Plugin principal.
// A human principal alone cannot represent that action. Headless calls never prompt; token/lifetime rules remain open.

// error.rs — codes FROZEN
pub enum OpError {
    UnknownOperation(OpId), NotExposed { op: OpId, caller: CallerKind }, Denied(DenyReason),
    InvalidArgs(ArgError), InputRequired(Vec<FieldName>), Unavailable(String),
    StaleDocument { captured: Revision, current: Revision }, Cancelled, OutOfBudget(BudgetKind),
    PluginMissing(Vec<MissingPlugin>), PluginFailed(PluginFailure), TooDeep, Internal(String),
}
impl OpError { pub fn code(&self) -> ErrorCode; pub fn exit_code(&self) -> u8; }
// exit: 0 done, 1 failed, 2 usage (UnknownOperation, InvalidArgs), 3 input required,
//       4 plugin missing or not permitted (PluginMissing, Denied of a Plugin grant), 5 cancelled
```

**Exposure rule** (architecture section 7):

| Marking | Exposure |
|---|---|
| Normal, exposed to the CLI or automation | Plugins too |
| Management | Gui, Cli, and Automation with `DocumentAndManagement` |
| Core | Gui, Script, Cli, Automation; never Plugin |
| Internal | Nested calls only |

The inventory check (P2-08) fails when an exposure contradicts this table.

### Provisional (may change until FigCraft adopts it in P8)

```rust
// dispatch.rs: the one sequence; apps implement the hooks, never the order.
pub trait OpApp {
    type Effect;                                   // the app's queued side effect type
    fn descriptor(&self, op: &OpId) -> Option<&OpDescriptor>;
    fn parse(&self, op: &OpId, args: Value) -> Result<ParsedArgs, OpError>;    // schema-validated, unknown fields refused, size/depth capped
    fn resolve(&self, inv: &Invocation, op: &OpId, args: &ParsedArgs) -> Result<Resolved, OpError>;  // bounded, no side effects
    fn authority(&self, inv: &Invocation) -> EffectiveAuthority;              // User/Cli full; Plugin from PluginAuthority
    fn available(&self, op: &OpId, r: &Resolved) -> Result<(), String>;       // the old `enabled`: a hint, never security
    fn run(&mut self, inv: &Invocation, op: &OpId, args: ParsedArgs, out: &mut Outbox<Self::Effect>) -> Result<Value, OpError>;
    fn txn(&mut self) -> &mut dyn TxnHost;
    fn apply_effects(&mut self, effects: Vec<Self::Effect>) -> EffectReport;
    fn record(&mut self, inv: &Invocation, op: &OpId, args: &Value, outcome: &Result<Value, OpError>); // journal, learning (User only), logs
}
pub fn dispatch<A: OpApp>(app: &mut A, inv: &Invocation, op: &OpId, args: Value) -> Result<Value, OpError>;
// lookup → exposure → parse → resolve → authorize → available → [Edit: begin or join txn] → run
//   → [outermost: commit (one step, label "<op label> · <origin>") | rollback] → effects after commit → events → record

pub trait TxnHost {
    fn begin(&mut self) -> TxnFrame;                               // snapshot: document, history, editor state, event mark
    fn commit(&mut self, frame: TxnFrame, label: UndoLabel) -> Revision;
    fn rollback(&mut self, frame: TxnFrame);
    fn revision(&self) -> Revision;
}
pub struct JobRecord { pub id: JobId, pub owner: Principal, pub captured: Revision, pub state: JobState,
                       pub progress: Option<f32>, pub outputs: Vec<Artifact>, pub error: Option<OpError> }
pub fn commit_check(captured: Revision, current: Revision) -> Result<(), OpError>;   // StaleDocument unless the op says it merges safely
```

**Nested calls.** A command that runs other commands (EffectCraft runs 98 such calls inside the engine) calls `dispatch` with `inv.child(op)`. The child:
- keeps the principal and joins the open transaction;
- **is authorized again**, which costs one constant-time check for user principals;
- skips the exposure check only for `Internal` operations.

So a Normal operation's implementation can never lend a plugin a Management operation.

## 4. Ordered PR list

| PR | Scope | Size | Files | Test that fails first | Mutants | Depends |
|---|---|---|---|---|---|---|
| P2-00 | `docs/design/ops.md`: the sequence, the exposure table, the nested-call rule, the error and exit-code table and the inventory encoding. Reviewed with the EffectCraft and FigCraft maintainers **before code** | S (docs) | `docs/design/ops.md` | — | — | P0 |
| P2-01 | Crate scaffold, `principal.rs`, `invocation.rs`, `clock.rs`; added to `[workspace.dependencies]` with `[lints] workspace = true` | S/M | `crates/ext-ops/**`, root `Cargo.toml` | `invocation::not_deserializable`: a `compile_fail` doctest deserializing `Invocation` from JSON. `invocation::child_keeps_principal_and_caps_depth`: a 33rd nesting returns `TooDeep` | "child() upgrades principal to User"; "depth cap off" | P2-00 |
| P2-02 | **Permission vocabulary in `ext-core`** (`permission.rs`, text form, parse and widen), plus `EffectiveAuthority::compute`. Lands early because manifest v2 (P1-10) and grants (P1-12) use it | M | `crates/ext-core/src/permission.rs`, `crates/ext-ops/src/authority.rs`, `docs/schemas/permissions.md` | `permission::unknown_text_refused` (`"document.delete:all"`, `"account.use:x"` in release 1); `authority::intersection_never_exceeds_grant` (proptest) | "widen() reports nothing"; "account family parseable" | P2-01 |
| P2-03 | Descriptors, markings, exposure derivation, and the reference app with one op of each kind and marking | M | `descriptor.rs`, `reference/` | `reference::plugin_cannot_reach_protected_ops`: a plugin invocation of `scripting.setPermissions` (Management), `undo` (Core) and `internal.sync` (Internal) returns `NotExposed`; the handler counter stays 0 | "Management exposed to Plugin"; "Internal callable top-level" | P2-01 |
| P2-04 | `resolve.rs`, `scope.rs`, `authorize.rs` and automation scope; `$N` batch references resolved boundedly and without side effects before `requires()` | M | as named | `authorize::explicit_target_outside_selection_denied`; `authorize::batch_ref_resolved_before_authorization` (step 2 targets `$1`, which resolves outside the selection → denied before step 1 runs); `authorize::attached_client_needs_management_approval` | "explicit target skips authorization" (the EffectCraft `dispatch.rs:11-20` shape); "availability treated as authorization" | P2-02, P2-03 |
| P2-05 | `dispatch.rs` sequence, `txn.rs`, `outbox.rs`; reference executor | M | as named | `txn::nested_edits_one_step` (two nested edits produce one undo step labelled `"Add many · Ink & Grain"`); `txn::failure_in_step_two_restores_all` (revision, history, outbox and events equal the pre-call values; the queued file write never runs) | "effects run before commit"; "nested call opens own txn"; "rollback skips history" | P2-04 |
| P2-06 | `job.rs`, `events.rs`, origin in records | M | as named | `job::stale_job_refuses_commit`; `job::queued_job_keeps_principal` (a job queued by a plugin and started later runs as that plugin, with the grant generation re-checked at start); `events::origin_chain_capped` (a plugin reacting to its own edit stops at chain length 8 with `TooDeep`) | "revision check skipped"; "job runs as User" | P2-05 |
| P2-07 | `error.rs` codes, exit codes, MCP error-data mapping; golden table | S | `error.rs`, `fixtures/errors.json` | `error::golden_codes`: every variant's code, exit code and MCP `data.code` equals the fixture | "PluginMissing exits 1" | P2-03 |
| P2-08 | Operation inventory encoding (canonical JSON, sorted keys) and breaking-change diff; shared with P4-01 for domain contracts | M | `inventory.rs`, `docs/schemas/inventory.md`, `fixtures/inventory/` | `inventory::unmarked_op_fails`; `inventory::breaking_change_needs_major` (removing an op, removing an arg field, narrowing a type or changing kind without a major bump is reported; adding an optional field is not) | "diff ignores removed op" | P2-03 |
| P2-09 | Broker traits and policy types (files, network, clipboard), with **no implementations**. Implementations land in `ext-host` before the capability is advertised (P3-08 and later, needed by P8) | S | `brokers/` | `brokers::network_policy_refuses_private_ranges` (a policy table test over IPv4 and IPv6 loopback, private, link-local, CGNAT and IPv4-mapped forms) | "loopback allowed" | P2-02 |

**Order.** P2-00 → P2-01 → P2-02 → {P2-03, P2-07} → P2-04 → P2-05 → {P2-06, P2-08} → P2-09.

**Changed from the reconciled plan:**
1. **The dispatcher sequence is shared code**, not only shared types. Each app implementing "exposure, authorize, availability" itself is how EffectCraft ended up with availability as a gate. A shared `dispatch()` with app hooks keeps the order identical everywhere. The trait is provisional until FigCraft adopts it.
2. **The permission vocabulary moves into `ext-core` and lands first** (P2-02), because the manifest and the store need it before ops does.
3. **Broker implementations move out of P2.** P2 defines the traits and the IP policy table. Implementations belong to the host, which runs plugin imports, and are needed only by session plugins (P8). Pilots ship transforms until then.
4. **App scripts get a principal** (`UserSurface::Script`). EffectCraft's `script_calls` counter (`dispatch.rs:45-55`) shows they are a real caller kind with narrower rights (no scripting-permission changes).

## 5. Data formats owned here

### 5.1 Operation inventory (`<app>/<contract>@<major>.<minor>.json`), shared with P4

```json
{
  "format": "craft-inventory", "format_version": 1,
  "contract": { "id": "effectcraft", "major": 1, "minor": 0 },
  "source": { "app": "effectcraft", "revision": "<full git sha>", "digest": "sha256:…" },
  "operations": [
    { "id": "layer.addItem", "kind": "edit", "marking": "normal",
      "exposure": ["gui", "script", "cli", "automation", "plugin"], "interaction": "none",
      "since": "1.0", "deprecated": null,
      "args": { "$schema": "https://json-schema.org/draft/2020-12/schema", "type": "object", "additionalProperties": false, "properties": { … } },
      "result": { … },
      "requires": ["document.edit:current-document"] }
  ]
}
```

- `requires` lists the **upper bound** of rights. The real check is `requires(resolved)` at run time, which may need fewer (for example `document.edit:selection`). `check` (P4) warns from this list; the runtime enforces.
- **Breaking changes** (need a major bump):
  - an operation removed;
  - kind changed;
  - an exposure narrowed;
  - a required argument added;
  - an argument removed or its type narrowed;
  - a result field removed;
  - the upper bound of `requires` widened. Widening it is breaking because consent sheets would understate.
- **Additive** (minor): an operation added, an optional argument added, a result field added.
- **Fixtures:** `fixtures/inventory/{base,added-op,removed-op,narrowed-type,widened-requires}.json`, each with the expected diff.

### 5.2 Error codes (`fixtures/errors.json`)

`unknown_operation`, `not_exposed`, `denied`, `invalid_args`, `input_required`, `unavailable`, `stale_document`, `cancelled`, `out_of_budget`, `plugin_missing`, `plugin_failed`, `too_deep`, `internal`. Each has its exit code and MCP `data.code`. Renaming one is breaking.

## 6. Test strategy

- **Reference app.** A list document with selection and revision, plus one op of every kind and marking:
  - `doc.list` (Query), `item.add` (Edit), `item.addMany` (nested Edit), `file.export` (Effect), `job.bake` (Job), `prefs.set` (Normal);
  - `scripting.setPermissions` (Management), `undo` (Core), `internal.sync` (Internal).

  Every exit criterion runs against it. EffectCraft's E3 tests (P6) repeat the same assertions on the real engine.
- **Property tests.**
  - `authorize` never returns `Allow` when `requires(resolved) ⊄ authority`.
  - A random tree of nested ops with random failures always leaves either the full commit or the exact prior state.
  - Origin chains never lose an element.
- **Hostile input.**
  - Arguments 10 MiB long, 10,000 deep, with duplicate keys or unknown fields: each returns `InvalidArgs`.
  - `$N` references that loop, point forward or point out of range.
  - Operation IDs with Unicode look-alikes.
- **Mutation.** One mutant per rule, listed in section 4.
- **Performance.** User principals get a constant-time path. Benchmark `dispatch` overhead against a direct call on the reference app, with an order-of-magnitude bound below 1 µs per call for `User(Gui)` on reference hardware. This protects slider drags (T04 amendment 8). Record the numbers; they are not a CI gate.

## 7. Risks and open questions

| # | Risk | Smallest experiment | Before |
|---|---|---|---|
| R1 | Re-authorizing nested calls costs too much in EffectCraft's heavy compound commands | Run `create_vr_environment` (`cmd/app_more.rs:549`, about 20 nested calls) through the reference dispatcher shim; time it | P2-05 |
| R2 | `schemars` output differs between versions, making inventory diffs noisy | Pin `schemars`; generate twice on two OSes; byte-compare | P2-08 |
| R3 | Apps' args are free text today (EffectCraft `params: &'static str`, `cmd.rs:105`), so typed schemas mean writing about 760 arg types | Measure on one EffectCraft module: derive `JsonSchema` for 20 commands and compare with `cmd/schema.rs:77` `params_schema`. If too costly, allow a transitional "schema from doc string" encoder that marks operations `typed: false` and **does not expose them to plugins until typed** | P6-E1 |

**Questions for maintainers:**
- **Q1.** Is a fresh MCP server process, started by an agent host, `Cli` (full, management included) or `Automation` (document only)? Recommended: `Automation`. A user who wants management runs `<app>-cli plugin …` themselves.
- **Q2.** Does a `Script` principal get management? Recommended: no, matching EffectCraft's rule that scripts cannot change script permissions (`cmd/settings.rs:37`).
- **Q3.** Origin chain cap: 8 is proposed. Nesting depth cap: 32 is proposed.
- **Q4.** Should operations without typed schemas be allowed to stay non-plugin-exposed (R3)? Recommended: yes, during migration, listed in the inventory as `typed: false`.

## 8. Effort and parallel work

| PR | Effort | Parallel with |
|---|---|---|
| P2-00 | S | P1-00..03, P3-00 |
| P2-01, P2-02 | S/M, M | P1-04..06 |
| P2-03, P2-07 | M, S | P3-01..03 |
| P2-04, P2-05 | M each (the transaction PR needs a scheduled specialist review) | P1-07..09 |
| P2-06, P2-08 | M each | P4-00, P6-E1 |
| P2-09 | S | P3-07 |

That is 9 PRs: 3 S and 6 M. P2 can start as soon as P2-00 is accepted. It needs only the ID types that already exist in `ext-core`.
