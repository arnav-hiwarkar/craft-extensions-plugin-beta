# Plugin system maps and workflows

These diagrams describe the current beta. They are editable Mermaid source, so
GitHub can render them beside the documentation. The [interactive overview](maps/overview.html)
is a self-contained companion that also opens offline. Runtime evidence and limits
are recorded in [STATUS](../../STATUS.md) and [testing](../../TESTING.md).

## Read by role

| Your task | Start | Detailed references |
|---|---|---|
| Use and manage plugins | [User guide](user-guide.md) | [Capabilities](capabilities.md), [test walkthrough](../../TESTING.md) |
| Make a plugin | [Author guide](author-guide.md) | [CLI](cli-reference.md), [UI](ui.md), [native protocol](native-protocol.md) |
| Support plugins in an app | [Adapter guide](adapter-guide.md) | [Architecture explanation](how-it-works.md), [framework handoff](framework-integration.md) |
| Publish or maintain listings | [Store and distribution](store-and-distribution.md) | [Maintainer guide](maintainer-guide.md), [catalogue procedures](https://github.com/arnav-hiwarkar/craft-plugins/blob/main/MAINTAINERS.md) |
| Find a feature or its limits | [Coverage matrix](coverage.md) | [EffectCraft contract](contracts/effectcraft.md), [PhotoCraft contract](contracts/photocraft.md), [FigCraft contract](contracts/figcraft.md) |

App and catalogue source links point to the private beta GitHub forks. The
[maintainer guide](maintainer-guide.md) links the catalogue policy and procedures.

## 1. Mind map

```mermaid
mindmap
  root((Craft plugins))
    People
      Users
        Discover and install
        Consent and enable
        Update pin rollback
        Projects and export
      Authors
        Manifest and contracts
        Wasm SDK
        Native protocol
        Forms panels tools
        Build test pack
      App developers
        One adapter per app
        Canonical commands
        Captured scopes and Undo
        Host UI renderer
      Maintainers
        GitHub vendor identity
        FOSS listing checks
        Signed metadata
        Reports and revocations
    Platform
      Core types and validation
      Store and shared Manager
      Host runtime
        Wasm sandbox
        Consented native helper
      Shared UI and CLI
    App capabilities
      EffectCraft
        Effects tools panels commands
      PhotoCraft
        Filters pixels layers picked files
      FigCraft
        Nodes layout text vectors
        No raster filter dispatch
    Compatibility
      Package semver
      Per contribution contracts
      ABI 1
      Forms schema 1
      UI levels 1.0 and 1.1
      Component fallbacks
    Evidence
      Source inspection
      Targeted tests and CLI smokes
      CI run and artifact revisions
      GUI and platform limits
    Deferred
      Theme and font activation
      Accounts and reviews
      Network clipboard private storage APIs
      TUF and native OS confinement
```

Vocabulary is not implementation: a permission spelling or a manifest kind does
not mean an app implements the corresponding operation. Consult the app contract.
Native full access can reach the operating system independently; it is not a Wasm
permission or a promise of additional host operations.

## 2. Ownership and trust boundaries

```mermaid
flowchart TB
    Author["Author source and GitHub Releases"] -->|"digest-pinned archive"| Store["Shared store and Manager"]
    Catalogue["Signed index and hashed package metadata"] -->|"verify signature, freshness and digests"| Store
    UI["App manager and launcher"] --> Store
    CLI["App CLI and craft-ext"] --> Store
    Store -->|"per-app version, enabled contributions, grants"| Adapter["One app adapter"]
    Adapter -->|"run on worker"| Wasm["Wasm guest: no WASI"]
    Adapter -->|"separate full-access consent"| Native["Native helper: separate process"]
    Wasm -->|"bounded host calls"| Auth["Known operation, declared rights, live grants, captured scope"]
    Native -->|"host calls use the same adapter checks"| Auth
    Auth --> Engine["Canonical engine and document owner"]
    Engine --> History["Commit or rollback document and history"]
    Native -.->|"full account access outside host transaction"| OS["Operating system and external effects"]
```

Source owners: `crates/ext-core`, `crates/ext-store`, `crates/ext-host`,
`crates/ext-ui`, `crates/ext-cli`, `crates/guest`, and each app's `crates/plugins`.
See [how it works](how-it-works.md#components).

## 3. Discover, install and consent

```mermaid
flowchart TD
    Browse["Search catalogue"] --> Verify["Verify signed index and freshness"]
    Verify --> Compatibility{"App contract and platform available?"}
    Compatibility -->|"no"| Explain["Explain unavailable contributions or offer compatible version"]
    Compatibility -->|"yes or partly"| Plan["Resolve package dependencies and selected release"]
    Plan --> Download["Download archive; verify size, digest and manifest"]
    Download --> Installed["Installed; nothing newly enabled or granted"]
    Installed --> Setup["Choose compatible contributions"]
    Setup --> Rights["Show host permission sentences and author reasons"]
    Rights --> Native{"Native package?"}
    Native -->|"yes"| Full["Ask separate full-computer consent for this version"]
    Native -->|"no"| Ready{"Required access granted?"}
    Full --> Ready
    Ready -->|"no"| Refuse["Keep disabled and show required access"]
    Ready -->|"yes"| Enable["Activate release and enable chosen contributions for this app"]
```

Installing a dependency does not grant its permissions. A partly compatible package
can expose usable contributions while naming the others and their reasons.
An optional permission is not silently granted. Native consent is per version.
See [user setup](user-guide.md) and [CLI setup](cli-reference.md).

## 4. One session event and one Undo step

```mermaid
sequenceDiagram
    participant User as User or app event
    participant Adapter as App adapter
    participant Worker as Guest worker
    participant Owner as Document owner
    participant Engine as Canonical engine
    User->>Adapter: Command, tool or current UI event
    Adapter->>Worker: Event plus captured invocation
    Worker->>Owner: Begin
    Owner->>Owner: Check live access and captured scope; open group
    Owner-->>Worker: Begin accepted
    loop Host operations
        Worker->>Owner: Call operation with bounded arguments
        Owner->>Owner: Validate invocation and authorize
        Owner->>Engine: Execute allowed canonical operation
        Engine-->>Owner: Result or recoverable operation error
        Owner-->>Worker: Typed result or error
    end
    Worker->>Owner: Finish with commit intent
    alt Successful event and still valid
        Owner->>Owner: Publish allowed staged files with validity checks
        Owner->>Owner: Final validation; commit document and history
        Owner->>Owner: Publish staged UI and overlays
        Owner-->>Worker: Finish accepted after finalization
    else Failure, cancellation, stale scope or revoked access
        Owner->>Owner: Roll back document, history and staged UI
        Owner-->>Worker: Finish failed
    end
```

File publication applies to PhotoCraft's picked-file operations. It precedes
accepting its document/UI commit. A later failure can leave earlier files on disk;
external effects are not covered by document Undo. Native helper activity outside
host calls has the same external-effect limitation. One recoverable operation
error may be handled by the guest; it does not by itself imply event failure.

Tool events share a gesture-wide group, so several successful updates still make
one Undo step. See [adapter transactions](adapter-guide.md#4-transactions-one-event-one-undo-step).

## 5. Host-call authorization

```mermaid
flowchart TD
    Call["Guest host call"] --> Known{"Known contract operation and valid arguments?"}
    Known -->|"no"| Error["Return typed error"]
    Known -->|"yes"| Live{"Invocation, contribution and live grant generation valid?"}
    Live -->|"no"| Abort["Refuse and invalidate work where required"]
    Live -->|"yes"| Required["Derive rights from arguments and host-captured scope"]
    Required --> Declared{"Calling contribution declares every required right?"}
    Declared -->|"no"| Error
    Declared -->|"yes"| Granted{"Live grants cover every required right?"}
    Granted -->|"no"| Error
    Granted -->|"yes"| Execute["Run canonical operation on the owner thread"]
    Execute --> Final["Check validity again at event or effect commit"]
```

Guest flags cannot reduce permission requirements. A broad document edit grant
can cover narrower read/edit scope, but the calling contribution must still declare
its requirements. Unknown operation IDs never become arbitrary CLI dispatch.
See [permissions](how-it-works.md#permissions-are-checked-on-every-host-call).

## 6. Shared UI and future framework

```mermaid
sequenceDiagram
    participant Guest as Plugin
    participant Owner as App owner
    participant UI as Shared renderer
    participant Framework as Host framework or egui fallback
    Guest->>Owner: Panel/window tree
    Owner->>Owner: Validate whole tree at declared UI level
    Owner->>Owner: Stage tree; publish on successful event commit
    Owner->>UI: Surface key, current revision and tree
    UI->>Framework: Draw host-owned controls and component fallbacks
    Framework-->>UI: Stable widget IDs and typed values
    UI-->>Owner: Events tagged with surface and drawn revision
    Owner->>Owner: Reject closed/stale/unauthorized replies
    Owner->>Guest: Current UI event
```

The plugin never selects or detects the rendering backend. The host installs the
framework runtime, aligns its egui revision and controls theme selection. Existing
UI 1.0 trees remain supported; named components require usable fallbacks.
Theme activation remains deferred. See [UI reference](ui.md) and
[framework integration](framework-integration.md).

## 7. Tool gesture lifecycle

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Active: Begin captures scope and history
    Active --> Active: Update validates and stages edits
    Active --> Committed: Commit succeeds and access is current
    Committed --> Idle: One Undo step
    Active --> RolledBack: Escape, failure, revocation or document replacement
    RolledBack --> Idle: Restore document and clear overlays
    Idle --> Idle: Pan, zoom or query
```

While a gesture owns document edits, unrelated edit commands must be refused.
Navigation and queries remain available. Closing a surface or replacing a document
cannot deliver stale edits. See [tools and overlays](adapter-guide.md#8-panels-windows-tools-and-overlays).

## 8. Updates, pins and rollback

```mermaid
flowchart TD
    Current["Active release"] --> Check["Refresh signed catalogue and check compatibility"]
    Check --> Pin{"Candidate satisfies the app's pin?"}
    Pin -->|"no"| Keep["Keep current release"]
    Pin -->|"yes"| Stage["Install candidate beside old release"]
    Stage --> Consent{"New rights or native version consent required?"}
    Consent -->|"yes"| Pending["Keep candidate pending; old release stays active"]
    Pending --> User["User reviews and confirms setup"]
    Consent -->|"no"| Activate["Activate compatible candidate"]
    User --> Activate
    Activate --> Reload["Reload only the affected app/plugin setup"]
    Reload --> Rollback["User may select older compatible release"]
    Rollback --> State{"Saved contribution state compatible?"}
    State -->|"yes"| Review["Review access and activate older release"]
    State -->|"no"| Preserve["Keep newer project data as placeholders and explain"]
```

A version choice does not erase newer project data or bypass access checks.
Opaque per-instance state migration is not implemented for EffectCraft.
See [version management](user-guide.md) and [store lifecycle](how-it-works.md#lifecycle).

## 9. Opening a project with missing plugins

```mermaid
flowchart TD
    Open["Open project"] --> Refs{"Recorded plugin contributions?"}
    Refs -->|"no"| Normal["Open normally"]
    Refs -->|"yes"| Check["Check installed release, contract, state and per-app setup"]
    Check --> Ready{"All recorded needs ready?"}
    Ready -->|"yes"| Continue["Open with available plugin contributions"]
    Ready -->|"no"| Prompt["Explain missing, disabled or incompatible needs"]
    Prompt --> Choice{"User choice"}
    Choice -->|"Download/setup"| Explicit["Perform explicit install and consent workflow"]
    Choice -->|"Open without"| Preserve["Keep project data and placeholders"]
    Preserve --> Preview["Preview may pass through missing rendering contributions"]
    Preview --> Export{"Final output needs a missing render contribution?"}
    Export -->|"yes"| Block["Refuse final export with a reason"]
    Export -->|"no"| Allow["Allow output independent of missing contributions"]
    Explicit --> Check
```

Opening a project does not silently download, enable or grant. EffectCraft and
FigCraft use project prompts. PhotoCraft currently bakes filters into pixels and
records no corresponding plugin requirements, so it has no missing-filter prompt.

## 10. Catalogue publication and revocation

```mermaid
flowchart TD
    Maker["Author with GitHub source and release assets"] --> Vendor["Register vendor identity and listing metadata"]
    Vendor --> Check["Validate OSI/public-source policy, identity, tag, assets and digests"]
    Check --> Human["One maintainer allows listing"]
    Human --> Build["Build package metadata and next serial index"]
    Build --> Sign["Sign index with private maintainer key"]
    Sign --> Publish["Publish metadata; archives stay on author Releases"]
    Publish --> Consumer["Consumers verify signatures, metadata hashes and archive digests"]
    Report["User report through GitHub issue form"] --> Triage["Maintainer triage"]
    Triage --> Revoke["Publish advisory, block-new or disable revocation"]
    Revoke --> Sign
    Consumer --> Severity{"Revocation severity"}
    Severity -->|"advisory"| Warn["Show warning"]
    Severity -->|"block-new"| Refuse["Refuse affected installs and new activation choices"]
    Severity -->|"disable"| Stop["Stop affected running work after refresh"]
```

Installation requires no Craft account. GitHub identity is used for vendor
registration and report submission. App accounts, reviews and ratings are deferred;
there is no implemented review-login workflow. The local test catalogue uses a
separate public test-key override limited to debug builds and file URLs.
See [store and distribution](store-and-distribution.md).
