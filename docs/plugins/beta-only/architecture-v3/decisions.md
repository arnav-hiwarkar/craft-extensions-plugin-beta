# Decisions

The owner answers each open decision by choosing one listed option, or by naming a different one. Every card gives the options, the recommendation and the reason, so the answer is one line. Once answered, the record moves to decided and the threads that depend on it proceed on that choice. Decided records below are binding, including the owner rule that this build has no plugin network [DEC-COM-no-plugin-network].

## Open: needs the owner

### DEC-COM-d-d: D-D docking web panels

Built state CefWindow: T6 panels float as slot-following palettes; migrate --to-t1 ports them to docking T1. All modes built and tested (O6, WEB1-4, S-WEB1, BD1); only docking parity waits.

- Option: web = CefDocked
- Option: web = CefDockedCompat
- Option: web = CefWindow (built state)

**Recommendation:** Recommended: CefDocked; narrower CefDockedCompat docks only imported UXP and CEP

**Why:** Incumbent panels dock; one frame of latency at 60 fps (S-WEB1).

### DEC-COM-d-k: D-K native hosts in public releases

craft-pshost, craft-aehost (AEGP, Panelator, AEIO) and the OFX bridge are clean-room and ship in beta; Adobe SDK header licensing review gates public releases.

- Option: Beta now, public after legal review (recommended)
- Option: Public releases now
- Option: Beta builds only

**Recommendation:** Recommended: beta now, public after review

**Why:** Review gates distribution only.

### DEC-COM-d-t: D-T CI hardware runners

Three self-hosted runners (REQ-COM-machine-class-rule), one job each: nightly hardware correctness suites, then performance at 20% thresholds; hw PRs run the subset. Hosted runners with device mocks close M4 and M5; the M6 beta exit waits on D-T.

- Option: Approve hw-mac, hw-win, hw-linux (recommended)
- Option: Hosted runners only (M6 beta exit blocked)

**Recommendation:** Recommended: approve the runners

**Why:** Hosted runners lack real GPUs, audio and displays.


## Decided

Each line gives the record id and its title, grouped by the part of the system it governs. The record holds the choice, the reason and what it affects.

### Common to all apps

- DEC-COM-acceptance-measurement: Acceptance targets measurement regime
- DEC-COM-ai-backends: AI backend placement
- DEC-COM-bd-defaults: BUILD_DEFAULTS
- DEC-COM-brush-engine-row: Brush engine row
- DEC-COM-build-order-sequencing: Build order is sequencing only
- DEC-COM-ci-capacity-plan: CI capacity plan
- DEC-COM-codeshift-built-at-m5: Codeshift built at M5
- DEC-COM-commit-rules: Commit conventions
- DEC-COM-cost-feedback-ema: cost-feedback-ema
- DEC-COM-craft-libs-patch-table: Apps take craft-libs through a patch table of path dependencies
- DEC-COM-cut-gates-required-jobs: CUT gates are required hosted checks
- DEC-COM-cut-ui-set-delivery: Cut UI set delivered before each cut
- DEC-COM-d-a: Plugin may hand links to the user's browser
- DEC-COM-d-b: D-B automatic selection writes, script prefs
- DEC-COM-d-b-net-pref: Script network prefs
- DEC-COM-d-c: D-C clipboard
- DEC-COM-d-e: D-E full access
- DEC-COM-d-f: D-F unconfined Windows
- DEC-COM-d-g: D-G native full access from any tier
- DEC-COM-d-h: Run-time generated WGSL under gpu.compile
- DEC-COM-d-i: Expression libraries evaluate only in ext-expr
- DEC-COM-d-j: FigCraft protocol 4
- DEC-COM-d-l: D-L presence
- DEC-COM-d-m: iOS wasmi in-process
- DEC-COM-d-n: Render contexts read snapshots with recorded hashes
- DEC-COM-d-o: D-O live writes
- DEC-COM-d-p: Direct OS clipboard for native helpers
- DEC-COM-d-r: Pending-vocabulary mechanism
- DEC-COM-d-s: D-S companions
- DEC-COM-d-u: D-U WFP
- DEC-COM-d-v: egui fork with on_wake and recreate_device
- DEC-COM-decision-6: Selection writes
- DEC-COM-decision-8: Preference groups
- DEC-COM-doc-access-snapshots: Reads and pure commands run on snapshots; commit is one owner hop
- DEC-COM-documenthost-size-runner: DocumentHost size and Runner
- DEC-COM-egui-fork: egui fork
- DEC-COM-error-codes-notfound: Out-of-scope returns -3 NotFound
- DEC-COM-execution-chosen-by-meta: Execution path chosen by Meta and doc size
- DEC-COM-ext-host-deleted: ext-host deleted; no in-process plugin execution
- DEC-COM-fixed-names: Fixed names: gate::admit, --safe-mode, craft-plugin-web
- DEC-COM-flagship-order: Flagship delivery order
- DEC-COM-fonts-activate-system-split: Fonts split
- DEC-COM-gate-order: Gates in order of need
- DEC-COM-gpu-tiers-all-built: All GPU tiers
- DEC-COM-hard-cut-gated-by-parity: Hard cut per subsystem gated by parity
- DEC-COM-heavy-routing-gate-step11: Heavy routing is gate step 11
- DEC-COM-helper-per-plugin: Helper per plugin
- DEC-COM-history-savepoints: History control savepoints
- DEC-COM-hooks-generic-intercept: Generic command interception hooks
- DEC-COM-input-monitor-not-never: input.monitor at L4
- DEC-COM-intercept-color-picker: app.intercept color.picker
- DEC-COM-interceptable-default-false: interceptable-default-false
- DEC-COM-jsxbin-clean-room: Clean-room jsxbin decoder
- DEC-COM-languages-all-day-one: All languages from day one; V8 on every desktop
- DEC-COM-lock-codeowner: lock-codeowner
- DEC-COM-m4-m6-week-replan: M4-M6 weeks assume provisional estimates
- DEC-COM-mac-scoped-checks-only: Mac runs only scoped checks
- DEC-COM-manifest-contribution-keys: Manifest keys for PhotoCraft contributions
- DEC-COM-merge-framework-base: Merge a6b9d9c into craft-libs plugin-beta
- DEC-COM-meta-cost-budgets: Meta Cost and DocSize with Cost::heavy
- DEC-COM-nativehostkind-uxpaddon: NativeHostKind UxpAddon
- DEC-COM-no-acg-on-jit-processes: No ACG on JIT
- DEC-COM-no-comments-deterministic-tests: No code comments, every rule has a deterministic test
- DEC-COM-no-jit-in-app-process: No JIT in app
- DEC-COM-no-plugin-block-save: No plugin can block save, export, quit or close
- DEC-COM-no-plugin-codecs: no-plugin-codecs
- DEC-COM-no-plugin-network: No plugin network
- DEC-COM-no-wit-component-model: no-wit-component-model
- DEC-COM-nothing-built-before-m-1: Nothing is built before M-1
- DEC-COM-one-engine-cef: CEF only
- DEC-COM-one-wgpu-host: One wgpu host
- DEC-COM-open-url-schemes: D-A open URL
- DEC-COM-ordered-cold-start: Ordered cold start
- DEC-COM-origin-by-value: origin-by-value
- DEC-COM-owner-decision-ids: Owner decision id registry
- DEC-COM-owner-frame-heavy-budget: owner-frame-heavy-budget
- DEC-COM-paged-texture: PagedTexture
- DEC-COM-panel-show-hide-commands: Synthesised panel show and hide
- DEC-COM-per-pr-deterministic-counters: Per-PR gates use deterministic counters only
- DEC-COM-permission-vocabulary-v2: Vocabulary v2
- DEC-COM-photoshop-contracts-shared-scriptui: photoshop.jsx and photoshop-uxp contracts
- DEC-COM-pkg-assets-not-copied: pkg-assets-not-copied
- DEC-COM-probation-device-not-app-queue: Probation device and queue
- DEC-COM-r1-r42: Draft contradictions resolved in architecture-v2 s21
- DEC-COM-rename-guest-native: Renames guest and native-sdk
- DEC-COM-render-present-separate: Render and present separate
- DEC-COM-render-session-partition-helpers: render-session-partition-helpers
- DEC-COM-repo-layout-branches: Repos and branches for plugin-beta work
- DEC-COM-restricted-profile-cut-order: Profile cut order
- DEC-COM-review-gate: Review gate for the five documents
- DEC-COM-script-provenance-entry: Script provenance
- DEC-COM-two-wasmtime-engines: Two Wasmtime engines
- DEC-COM-txn-modal-everywhere: txn.modal for every app; user events promote to modal
- DEC-COM-ui-egui-single-toolkit: Every app UI toolkit is egui
- DEC-COM-ui-tiers-t1-t6: T1-T6 built with one layout engine
- DEC-COM-undeclared-agent-only: Undeclared commands are agent-only
- DEC-COM-undeclared-commands-zero: Beta apps reach zero UNDECLARED before adapters ship
- DEC-COM-v1-1: Canonical ops plus registry bridge behind one gate
- DEC-COM-v1-10: Retained v1 decisions
- DEC-COM-v1-13-14-15-18: Branches, app order, GPU crates, hard cut
- DEC-COM-v1-2: Command metadata next to each command
- DEC-COM-v1-24: Host model packages
- DEC-COM-v1-31: Suite replacement rule trust-boundary amended
- DEC-COM-v1-32: TUF now
- DEC-COM-v1-34: Agent access
- DEC-COM-v1-4: Widening review
- DEC-COM-v1-7: No WIT; ParamSchema generates bindings
- DEC-COM-web-panel-browsers-partitions: Auxiliary panel browsers and CEF
- DEC-COM-worker-budget-by-memory: Workers by memory
- DEC-COM-ws12-spike-first: WS12 spike slice before tasks 4-5
- DEC-COM-xdoc-ec-v2: effectcraft.md cross-doc changes
- DEC-COM-xdoc-pc-v2: photocraft.md cross-doc changes v2
- DEC-COM-xdoc-pc-v22: photocraft.md cross-doc changes v2.2
- DEC-COM-zygote-pool: Zygote pool

### Security

- DEC-SEC-cef-only-web: CEF only for web windows
- DEC-SEC-devices-broker-usb: Devices broker USB
- DEC-SEC-gpu-residual: GPU driver residual
- DEC-SEC-hardware-consent-only: Setup-time consent
- DEC-SEC-launch-path-5ms: macOS launch path
- DEC-SEC-lpac-com-native-ui: D-P native UI COM and pasteboard
- DEC-SEC-os-open-allowlist: os.open allow-list
- DEC-SEC-pinned-shell-argv: Shell commands
- DEC-SEC-pshost-8bam-handles: 8BAM handles
- DEC-SEC-pshost-notice-contexts: pshost notices
- DEC-SEC-residual-full-access-total: Accepted residuals
- DEC-SEC-scope-violation-minus-3: Scope -3
- DEC-SEC-threads-shared-timing: Shared threads
- DEC-SEC-web-failure-no-system-webview: S-WEB1 failure never restores
- DEC-SEC-web-shared-engine-isolation: Shared engine, separate renderers
- DEC-SEC-win-lpac-profile: pshost LPAC

### EffectCraft

- DEC-EC-aehost-clean-room: AE host written from public structure layouts
- DEC-EC-aehost-windows-same-milestone: Windows aehost built with macOS
- DEC-EC-checkpoints-plugin-account: Sim checkpoints charged to plugin accounts
- DEC-EC-ease-257-samples: Plugin ease sampled 257 times
- DEC-EC-ec-cep-node-compat: EffectCraft CEP via node-compat
- DEC-EC-ec-fonts-install: EffectCraft font install
- DEC-EC-ec-memcap-memaccount: memcap to MemAccount
- DEC-EC-ec-meta-cost-routing: EffectCraft Meta.cost routing
- DEC-EC-ec-scriptui-psbackend: EffectCraft ScriptUI and PsBackend
- DEC-EC-ec-taint-pointer: EffectCraft taint rule points to PC
- DEC-EC-expr-memo-app-cache: Expression memo is app cache
- DEC-EC-expr-provenance-evaluator: Provenance decides the expression evaluator
- DEC-EC-fx-apply-no-permission: fx.apply free
- DEC-EC-hard-cut-legacy-plugins: Hard cut for legacy plugin systems
- DEC-EC-js-on-worker-v8: JavaScript runs on worker JIT engine
- DEC-EC-gt3-recorded-dag-validation: GT3 validates a recorded command list, not a live API
- DEC-EC-native-effects-restricted-default: Native effects restricted
- DEC-EC-no-gpu-side-helper-wait: App queues never wait on helper-signalled values
- DEC-EC-one-gate: One gate over canonical ops plus app.run
- DEC-EC-script-corpus-hashes-first: Corpus hashes before script changes
- DEC-EC-script-folders-hard-cut: Script folders replaced by extendscript@1 packages
- DEC-EC-socket-to-net-sink: Socket to net sink
- DEC-EC-t5b-default-panelator: Panelator docks as T5b by default
- DEC-EC-user-scripts-to-worker: Scripts in worker
- DEC-EC-worker-per-version: Sandboxed worker per package version

### PhotoCraft

- DEC-PC-8bf-hosting-built: 8bf hosting in craft-pshost, superseding v1 decision 25
- DEC-PC-accepted-coverage-lacks: Coverage gaps fail loudly
- DEC-PC-accepted-gpu-replay-adapters: GPU replay adapters
- DEC-PC-accepted-handoff-risk: Vendor handoff
- DEC-PC-accepted-uxp-fidelity: UXP layout fidelity
- DEC-PC-adapter-file-list: Adapter file list size
- DEC-PC-brush-engines-gpu-emit: Brush engines may emit gpu
- DEC-PC-common-documenthost-txn: Common DocumentHost transactions
- DEC-PC-compat-file-execute-mapping: Compat process and file mapping
- DEC-PC-cost-routed-heavy-calls: Cost-routed heavy calls
- DEC-PC-craft-pshost-not-exposed: pshost internal
- DEC-PC-devices-usb-webusb: devices.usb with WebUSB semantics
- DEC-PC-dispatch-allowed-app-scripts: Dispatch commands under app.scripts
- DEC-PC-durable-output-default: Durable output defaults to full resolution
- DEC-PC-exposure-ui-window-dialog: Window and dialog commands visible
- DEC-PC-fate-atn-cmds-psdesc: v2 fate: atn-cmds-psdesc
- DEC-PC-fate-automate-provenance: v2 fate: automate-provenance
- DEC-PC-fate-cmd-overlay: v2 fate: cmd-overlay
- DEC-PC-fate-engine-lib-fields: v2 fate: engine-lib-fields
- DEC-PC-fate-engine-plugin-filters: v2 fate: engine-plugin-filters
- DEC-PC-fate-engine-plugin-ops: v2 fate: engine-plugin-ops
- DEC-PC-fate-engine-plugin-sessions: v2 fate: engine-plugin-sessions
- DEC-PC-fate-engine-plugin-tests: v2 fate: engine-plugin-tests
- DEC-PC-fate-jobs-authorization: v2 fate: jobs-authorization
- DEC-PC-fate-plugins-contract-rs: v2 fate: plugins-contract-rs
- DEC-PC-fate-plugins-document-thread: v2 fate: plugins-document-thread
- DEC-PC-fate-plugins-filter-rs: v2 fate: plugins-filter-rs
- DEC-PC-fate-plugins-lib: v2 fate: plugins-lib
- DEC-PC-fate-plugins-resources-rs: v2 fate: plugins-resources-rs
- DEC-PC-fate-plugins-runtime-rs: v2 fate: plugins-runtime-rs
- DEC-PC-fate-plugins-session-filters: v2 fate: plugins-session-filters
- DEC-PC-fate-print-cmds-sink: v2 fate: print-cmds-sink
- DEC-PC-fate-session-execute: v2 fate: session-execute
- DEC-PC-fate-ui-panels-registry: v2 fate: ui-panels-registry
- DEC-PC-fate-ui-plugin-form: v2 fate: ui-plugin-form
- DEC-PC-fate-ui-plugins-ui: v2 fate: ui-plugins-ui
- DEC-PC-gpu-gt3-built: GPU GT3 programs built
- DEC-PC-javascript-compat-built: JavaScript compat built
- DEC-PC-kept-behaviours-before-deletion: Kept behaviours tested first
- DEC-PC-native-helper-profile-ui-window-shortcuts: Native-UI helper profile
- DEC-PC-never-list-pc-adds-none: PhotoCraft never list and pshost profile
- DEC-PC-one-gate-automation-plugins: One exposure function for automation and plugins
- DEC-PC-photoshop-generator-shim: Generator inside photoshop-compat
- DEC-PC-print-fonts-process-permissions: Print, fonts and reveal permitted
- DEC-PC-provenance-routing-plugin-scripts: Provenance of plugin-written scripts
- DEC-PC-smart-filter-shared-tile-tree: Smart-filter output shares tiles
- DEC-PC-undo-manages-history-control: Undo::Manages under history.control
- DEC-PC-uxp-page-runtime-cef: UXP pages in CEF
- DEC-PC-web-build-plugins-run: Web build plugins in a Web Worker
- DEC-PC-workstream-updates: Workstream updates for PhotoCraft

### FigCraft

- DEC-FC-build-defaults: Switchable decisions behind BUILD_DEFAULTS
- DEC-FC-cost-from-calibration: Cost drafts from census, k fitted per family
- DEC-FC-d-b: Decided D-B: automatic selection.write
- DEC-FC-d-g: Decided D-G: native full for Community authors
- DEC-FC-delete-blocking-bridge: Delete the 7 s blocking Bridge::ask
- DEC-FC-fc-meta-cost-pernode: FigCraft Meta.cost PerNode
- DEC-FC-fc-permission-vocab-generated: Generated permission page
- DEC-FC-fc-theme-token-source: One theme token source
- DEC-FC-first-adapter: FigCraft integrates first
- DEC-FC-full-meta-coverage: 100% Meta coverage with real schemas and Cost
- DEC-FC-gpu-lazy-job-device: Lazy job device
- DEC-FC-hard-cut-session-runtime: Hard cut of figcraft.session@1.2
- DEC-FC-hard-cut-session1: Hard cut of figcraft.session@1.x
- DEC-FC-image-keys-blake3: BLAKE3 image keys
- DEC-FC-no-locked-error: -12 Locked unused in FigCraft
- DEC-FC-no-owner-queue-cap: No owner-queue cap; wait time charged to caller
- DEC-FC-nodes-refs-strings: Node refs are strings space:n
- DEC-FC-server-owns-plugin-data: Server owns plugin data
- DEC-FC-three-layers-one-gate: Canonical, adapter and bridge layers share one gate

