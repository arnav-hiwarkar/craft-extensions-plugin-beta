# How craft-libs is consumed

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Historical discussion archive, captured on 2026-11-08. This preserves technical requests and substantive responses; routine progress updates and tool logs are omitted. Local checkout links are normalized. Later decisions and source corrections take precedence. Start with [the current documentation index](../README.md).

## User request

how does craft-libs work rn and how will all the apps use stuff made here, stuff like plugin management etc

## Research response 1

`craft-libs` is the shared Rust code that the ArtCraft apps build into themselves. Each app chooses the libraries it needs and connects them to its own editor.

Today, it contains individual packages called crates under `crates/`: settings storage, keyboard shortcuts, fonts, dialogs, PDF support, shared UI components, and more. They can depend on each other, but they must not depend on an app. That keeps them reusable. The [README lists the existing libraries](https://github.com/storytold/craft-libs/blob/fd5e2feff6ea25bed371a99cc732b3968d621/README.md#L26).

**How an app uses something here**

An app declares a dependency in its `Cargo.toml`, pointing to a specific Git commit:

```toml
craft-settings = {
    git = "https://github.com/storytold/craft-libs",
    rev = "<commit>"
}
```

Cargo downloads that revision and compiles the library into the app. The app then calls its Rust functions.

This is already happening. [FigCraft’s dependencies](https://github.com/storytold/figcraft/blob/0e86f3597a7443b15c93ec3ab2091202f8802c95/Cargo.toml:36) include shared settings, fonts, menus, and UI. [VectorCraft’s keyboard integration](https://github.com/storytold/vectorcraft/blob/065a646799f1f20f5454845d6648e09a5252c452/crates/ui-egui/src/keymap.rs:1) uses `craft-keymap` for shortcut machinery while keeping VectorCraft’s commands in VectorCraft.

Changes here reach users through an explicit process:

1. Merge the shared library change.
2. Update the consuming apps’ dependency revisions and any integration code.
3. Test, build, and release those apps.

**Merging here does not automatically update every app.** The local app checkouts already use different revisions.

**Where plugin management would fit**

There is no suite-wide plugin-management crate here yet. Some apps already have their own plugin systems. [EffectCraft’s implementation](https://github.com/arnav-hiwarkar/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/crates/plugin/src/lib.rs:1) loads WebAssembly effects that process pixels. [FigCraft’s implementation](https://github.com/storytold/figcraft/blob/0e86f3597a7443b15c93ec3ab2091202f8802c95/crates/plugin/src/lib.rs:1) supports plugins with commands, permissions, panels, and widgets. Those are different interfaces.

A shared system could put the reusable parts here:

- Plugin metadata, versions, and compatibility checks.
- Installation, updates, removal, and installed-plugin discovery.
- Permission records and enable/disable settings.
- Common execution limits and runtime machinery where compatible.
- A reusable plugin-manager panel.

Each app would still supply the connection to its editor: PhotoCraft’s pixels and layers, VectorCraft’s paths, EffectCraft’s effects and timeline. The shared library would expose interfaces for those connections rather than knowing the apps’ internal document types.

We already have a useful precedent: [the font manager](https://github.com/storytold/craft-libs/blob/fd5e2feff6ea25bed371a99cc732b3968d621/crates/font-manager/src/lib.rs#L3) provides both shared functionality and an optional complete UI panel. Its host supplies the catalog and store, saves settings, and places the panel inside its own interface. Plugin management could follow that pattern.

Two distinctions matter for the eventual design. Sharing the manager does not automatically make every plugin work in every app; compatibility needs an explicit contract. And sharing the code does not automatically share installed plugins between apps; that requires an agreed storage location and rules for concurrent access.

So we can build the common plugin system once here, then integrate it into each app while preserving what makes each editor different.

