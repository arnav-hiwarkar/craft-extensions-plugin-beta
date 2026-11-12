# Common management and app integration

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Historical discussion archive, captured on 2026-11-08. This preserves technical requests and substantive responses; routine progress updates and tool logs are omitted. Local checkout links are normalized. Later decisions and source corrections take precedence. Start with [the current documentation index](../README.md).

## User request

So we'll have one common integration for plugins which will have all the basic and core features like plugin data, updates, installations, permissions, and then we'll have specific integrations, right? So what will we need for app-specific integrations to work commonly, state this work properly through the entire thing? So for example, photograph might have its own integration, and like an update push to it to work with the plugin manager we are making in Craft Labs, same thing will. Same thing will happen for effect craft or fake draft and stuff like that. So how what exactly will these apps need in their changes do it and what exactly is a plugin in a project so like tell me in terms of effect graph when someone makes a plugin, how do we get a proper plugin thing to work with the core application? Someone can install a plugin in effect craft. What should that plugin be? What should it have? How do these plugins work and properly work with the application to add functionality and what are some security concerns which we'll need to be addressing for them

## Research response 1

Yes. We would have **one shared plugin system, with an adapter in each app**. That adapter connects plugins to the app’s actual functionality.

There are three separate contracts to get right:

- The **package contract** describes what is being installed, its version, compatibility, permissions, and files.
- The **execution contract** defines how plugin code runs and exchanges data with the app.
- The **editor contract** defines what the plugin can do: process pixels, edit paths, add commands, show panels, and so on.

Sharing installation alone would cover only the first part.

I also found that EffectCraft already has a substantial [Plugin Library implementation](https://github.com/arnav-hiwarkar/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/crates/engine/src/system/library.rs:1), including a catalog, installation, updates, removal, checksums, and missing-effect discovery. We should use that existing behavior as a starting point for extraction and migration.

**What each app would need to change**

Each app would need to implement the same integration responsibilities, while supplying its own editor operations:

| Integration responsibility | What the app supplies |
|---|---|
| Identify itself | App ID, version, platform, supported plugin APIs and capabilities |
| Connect the manager | Initialize the shared manager, expose its UI and commands, report progress and errors |
| Register extensions | Add a plugin’s effects, commands, panels, or importers to the appropriate app registry |
| Handle plugin requests | Translate permitted operations into the app’s existing commands and rendering interfaces |
| Enforce access | Check permissions against the current document, selection, files, and user action |
| Preserve editor behavior | Integrate edits with undo, dirty state, saving, rendering, and cache invalidation |
| Handle lifecycle | Startup loading, disabling, failed plugins, pending updates, and restart requirements |
| Preserve compatibility | Migrate existing installations and retain plugin IDs used by saved projects |

For example, PhotoCraft would provide access to its image and layer operations. EffectCraft would provide its effect rendering interface. FigCraft would provide its supported document commands and panel interactions.

We should provide shared integration tests that every adapter runs. This would check that denied operations remain denied, failed edits leave the document intact, and save/reopen preserves plugin state.

Once those adapters exist, a new compatible plugin can be installed **without releasing a new app build**. Adding an entirely new kind of capability, such as giving effects access to another layer’s pixels, can require a new API and app update.

**What a plugin actually is**

A plugin is executable code, plus enough information for the host to understand and safely invoke it.

For a proposed common package, an author would supply:

- A stable package ID, name, version, description, author, and licence.
- The compiled plugin module, initially a WebAssembly `.wasm` file.
- Supported apps and versioned APIs.
- The functionality it contributes, each with a stable ID.
- Requested permissions.
- Controls and parameter definitions.
- Any supported assets or saved-state schema.

The distribution catalog would additionally identify the downloadable release, its size, checksum, and publisher provenance. The package description and embedded module metadata must agree.

The exact filenames and schema are still design decisions. We should distinguish **package versions**, **execution API versions**, and **saved-state versions**; they change for different reasons.

**An EffectCraft example**

Suppose someone creates “Glow Rings,” an effect that transforms a layer’s pixels.

EffectCraft already supports this general workflow. A WebAssembly effect exports functions for reporting its API version and metadata, allocating a buffer, and rendering. The current interface is documented in its [plugin contract](https://github.com/arnav-hiwarkar/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/docs/plugins.md:173).

The author describes controls such as radius, intensity, and colour, then implements the rendering calculation. The plugin receives pixels, evaluated parameter values, layer time, and preview scale.

After installation:

1. The manager verifies the release and compatibility, then records the installed package.
2. EffectCraft’s adapter loads the module and validates its effect description.
3. The adapter registers the effect with EffectCraft’s effect registry.
4. The effect appears in the effects browser and menu.
5. The user applies it to a layer. EffectCraft creates ordinary effect properties for its controls.
6. During rendering, EffectCraft evaluates keyframes and expressions, passes the resulting values and pixels to the plugin, and receives processed pixels.
7. Saving the project preserves the effect ID and parameter values.

That registration already turns plugins into ordinary effects in [EffectCraft’s registry](https://github.com/arnav-hiwarkar/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/crates/effects/src/plugin.rs:467). The plugin author does not need to implement EffectCraft’s timeline, keyframe editor, or project format.

But this interface only enables the operations it exposes. A pixel effect cannot automatically add a new dockable panel or manipulate compositions. Those require separate, versioned extension interfaces.

A project should store a **reference to the plugin and its instance settings**, rather than automatically executing code embedded in the project. I would also record the required version and state schema, preserve missing-plugin settings, and offer an explicit resolution flow. Opening someone’s project should never silently download and execute its dependencies.

**The security boundaries we need**

WebAssembly is a sensible starting point because it separates module execution from the host. Access outside the module depends on the interfaces the host exposes. Those interfaces therefore become part of our security boundary. [WebAssembly’s security model](https://webassembly.org/docs/security/)

The main concerns are:

- **Permissions must be enforced on every operation.** A permission declaration is only a request. The host must check actual grants before executing a command. UI, CLI, scripting, and automation paths must use the same checks.
- **Keep grants narrow.** Processing the supplied frame should not imply access to every document, arbitrary files, networking, shell commands, or credentials. Grants should be scoped to the app and, where appropriate, the document or user-selected file.
- **Bound resource use.** Limit execution, memory, concurrency, output sizes, callbacks, and storage. Fuel limits count execution work; they are not a complete deadline or cancellation system. `wasmi` provides [fuel metering](https://docs.rs/wasmi/latest/wasmi/struct.Config.html#method.consume_fuel), but host operations need their own limits.
- **Protect installation and updates.** Validate paths and extraction sizes, reject traversal and dangerous links, verify trusted release metadata, and install transactionally. A checksum detects disagreement with the catalog; it does not establish that the catalog or publisher is trustworthy.
- **Treat permission increases as new decisions.** Updating a plugin must not silently grant additional access.
- **Protect project integrity.** Validate returned edits and render data. Apply document changes through transactions and undo. Failures should preserve the previous document or input frame.
- **Preserve reproducibility.** Keep versions needed by projects where practical. Changing a plugin version can change rendered output. Running sessions should retain a consistent version rather than switching midway through a render.

EffectCraft already rejects all imports for its WebAssembly effects, meters execution, and configures a 1 GiB linear-memory limit in its [runtime implementation](https://github.com/arnav-hiwarkar/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/crates/plugin/src/wasm.rs:42). That is useful protection, but it is not a total application memory budget or proof that crashes are impossible. Shared budgets and runtime maintenance still matter.

My recommendation is to start by sharing the package lifecycle and adapting EffectCraft’s existing WebAssembly effects. Preserve its existing IDs and installations, use explicit restart requirements initially, and then add further extension interfaces against concrete PhotoCraft and FigCraft use cases.

