# Plugins — user guide

Plugins add features to EffectCraft, PhotoCraft, FigCraft and the other ArtCraft apps.
They are made by other people. This guide shows how to find them, install them and stay in control.

## 1. What plugins are

A plugin can add:
- **Effects and filters** — new looks you can apply to your work.
- **Panels and windows** — extra controls, usually in their own panel.
- **Commands** — new entries in menus.
- **Tools** — new ways to work on the canvas.
- **Data add-ons** — presets, LUTs and fonts you can use like built-in ones.

Most plugins run in a **sandbox**. The app decides what a plugin can touch, and the plugin can only do what you allow. It cannot read your files or use the network unless you grant that.

Some plugins are **Native**. A Native plugin runs as a normal program, so it can do anything you can. The app labels it **"Native: full access to your computer"** and asks for your consent separately.

## 2. Finding plugins

Every app has a **Plugins** window. Open it from the menu. It has Browse, Installed, Updates and Problems tabs. You do not need anything else to use plugins.
The **ArtCraft launcher** (optional) has a Plugins tab that manages plugins for all your apps in one place.

You can also search from a terminal:

```text
effectcraft-cli plugin search halftone
```

Each result shows a badge:
| Badge | Meaning |
|---|---|
| **Official** | Made by the ArtCraft team. |
| **✔ Verified vendor** | The author's identity was checked by a maintainer. |
| **Community** | Listed, but the author is not verified. |
| **Not reviewed** | A local file you installed yourself. |

Results also show the project's stars and the date they were counted.

## 3. Installing

**Installing never turns anything on.** You always get a setup sheet next. It shows:

- The contributions you can turn on. Pick the ones you want.
- The **required permissions**. You must allow these to use those contributions.
- The **optional permissions**. These are off until you turn them on.
- The author's reason for each permission, quoted under the app's own sentence.
- Warnings, such as **"Could send your work to …"**.

For a Native plugin the sheet shows the full-access warning and asks for your consent.

To install a plugin you downloaded as a file, use the Plugins window or:

```text
effectcraft-cli plugin install-file ./ink-and-grain.craftplugin
```

A local file is always treated as **Not reviewed**, even if the same plugin is listed in the catalogue.

## 4. Permissions

| Permission | What it allows |
|---|---|
| `document.read:selection` / `:document` / `:all` | Read your selection, the current document, or any open document. |
| `document.edit:selection` / `:document` / `:all` | Change your selection, the current document, or any open document. |
| `files.read:picked` / `files.write:picked` | Read files, or save to files, that you pick in a file dialog. |
| `network:<host>` | Connect to one named https host. |
| `clipboard.read` / `clipboard.write` | Read the clipboard, or write to it. |
| `ui.window` | Open its own floating window. |
| `app.events` | Be told about app events, such as opening a document. |

Panels, menus, commands, effects, a plugin's own storage and logging need no permission.

You can change or revoke any permission at any time in the **Installed** tab. Revoking stops work that is running: queued work is cancelled and late results are thrown away. Anything already sent over the network cannot be called back.

## 5. Versions

Updates always ask. Nothing updates on its own.
If an update asks for **more permissions**, it waits for your approval. The version you already have keeps working until you accept.

You can install an older listed version, **pin** a version so it stays, and **roll back** at any time. If a version does not fit your app, you see a clear message, such as:

> version 2.x needs raster.filter 2; version 1.4.0 works here — install 1.4.0?

## 6. Projects with plugins

A project can remember which plugins it uses. **Opening a shared project never downloads or turns on anything.**

If the project needs plugins, you get one prompt: **"This project uses 3 plugins"**, with the permissions each one needs and three choices: **Download**, **Enable** or **Skip**.

If a plugin is missing, your data is kept exactly as it was. The feature shows a placeholder, and the preview passes through with a badge. **Final export refuses** until the plugins that render (effects) are available. Missing panels or commands never block export.

## 7. Problems and safety

The **Problems** tab lists plugins that failed, paused, do not fit, or are blocked, with the reason and a fix button.

If an app will not start because of a plugin, start it with plugins off:

```text
EFFECTCRAFT_SAFE_MODE=1 effectcraft
PHOTOCRAFT_SAFE_MODE=1 photocraft
```

In safe mode no plugin code runs. The Plugins window still works, so you can disable or remove the plugin. Your project data is kept. The other apps use the same pattern.

**Report plugin** opens a pre-filled issue for the maintainers. You can report from the Plugins window, the launcher, or `plugin report`.

The catalogue keeps a **kill list**. If a plugin is withdrawn for safety, it is turned off in every app on the next refresh, and it stays off. This applies to local installs of the same plugin too.

## 8. CLI cheat sheet

Commands are the same in every app. Replace `effectcraft-cli` with your app's CLI.

| Task | Command |
|---|---|
| Search | `effectcraft-cli plugin search <words>` |
| Show details | `effectcraft-cli plugin info <id>` |
| Install | `effectcraft-cli plugin install <id>` |
| Turn on | `effectcraft-cli plugin setup <id> --enable <contributions> --grant <permission>` |
| List permissions | `effectcraft-cli plugin permissions <id>` |
| Grant or revoke | `effectcraft-cli plugin grant <id> <permission>` / `plugin revoke <id> <permission>` |
| Check for updates | `effectcraft-cli plugin updates` |
| Update | `effectcraft-cli plugin update <id>` |
| Roll back | `effectcraft-cli plugin rollback <id> <version>` |
| Pin or unpin | `effectcraft-cli plugin pin <id> <version>` / `plugin unpin <id>` |
| Remove | `effectcraft-cli plugin remove <id>` |
| Report | `effectcraft-cli plugin report <id>` |
| Check a project | `effectcraft-cli plugin project <file>` |

Plugins are made by their authors. ArtCraft lists them but does not build, host or guarantee them.
