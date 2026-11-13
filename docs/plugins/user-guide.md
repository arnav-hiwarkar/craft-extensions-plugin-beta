# Plugins: user guide

Plugins add features to EffectCraft, PhotoCraft and FigCraft. Other people make them. This guide
shows how to find, install, set up and control them.

## 1. What a plugin can add

| Kind | What you get | EffectCraft | PhotoCraft | FigCraft |
|---|---|---|---|---|
| Effect | A new effect or filter | Yes | Yes | No |
| Command | A new menu command | Yes | Yes | Yes |
| Panel | A panel with the plugin's controls | Yes | Yes | Yes |
| Tool | A new way to work on the canvas | Yes | Yes | Yes |
| Automation (event hook) | Reacts when something happens in the app | Yes | Yes | Yes |
| Presets and LUTs | Data you use like built-in presets and LUTs | Yes | Yes | No |

Fonts and themes can be installed but are not used by any app yet.

Most plugins run in a **sandbox**. A sandboxed plugin can only use what the app offers it, and only
with the permissions you grant. It cannot run programs, use the network or the clipboard, or read
files you did not pick.

Some plugins are **native**. A native plugin runs a program on your computer. It can do anything you
can do. The app labels it **"Native: full access to your computer"** and asks for separate consent.

## 2. Finding plugins

Open the plugin manager:

| App | Menu |
|---|---|
| EffectCraft | **Plugins ▸ Manage Plugins...** (opens Settings ▸ Plugins) |
| PhotoCraft | **Plugins ▸ Plugins…**, or **Edit ▸ Preferences ▸ Plugins…** |
| FigCraft | **Plugins ▸ Manage plugins…** |
| ArtCraft Launcher | **Plugins** in the header (all apps in one place) |

The manager has four tabs: **Browse**, **Installed**, **Updates** and **Problems**, and a
**Refresh catalogue** button.

In **Browse**, type in **Search plugins**. You can filter by kind, tick **Works in this app**, and
choose **Include native plugins**. Each result shows its vendor and a badge:

| Badge | Meaning |
|---|---|
| Official | Made by the ArtCraft team |
| ✔ Verified vendor | A maintainer checked the author's identity |
| Community | Listed in the catalogue; the author is not verified |
| Not reviewed (local file) | You installed it from a file |
| Development | Installed by a plugin author's development tools |

The row also shows the project's stars and the date they were counted, and
"Native: full access" for native plugins.

From a terminal (replace `effectcraft-cli` with your app's CLI):

```sh
effectcraft-cli plugin search halftone
effectcraft-cli plugin info io.github.arnav-hiwarkar.halftone
```

Plugins are made by their authors. ArtCraft lists them but does not build, host or guarantee them.

## 3. Installing

**Installing never turns anything on.**

In **Browse**, press **Find compatible releases**, choose a version and press **Install**. The
manager asks **"Install {id}?"** and lists anything else it installs ("Also installs: ...").
Press **Confirm install**. The setup sheet opens next.

From a terminal:

```sh
effectcraft-cli plugin install io.github.arnav-hiwarkar.cross-review
effectcraft-cli plugin install-file ./cross-review.craftplugin
```

A file you install yourself is always marked **Not reviewed (local file)**. Installing from a file
is available from the CLI only.

## 4. The setup sheet

The sheet is titled **"Set up {plugin}"**. Nothing is enabled until you press **Confirm setup**.

1. **Contributions.** Tick the parts you want. A part this app cannot run shows
   "Unavailable: {reason}". You must choose at least one.
2. **Permissions.** Each permission is a checkbox with the app's own sentence, marked
   **(required)** or **(optional)**. If the author gave a reason, it appears below in italics as
   `Author: "..."`. **Confirm setup** stays disabled until every required permission is ticked.
   Optional permissions start off; the plugin works without them, with less.
3. **Warnings**, if any.
4. **Native consent.** For a native plugin, a red checkbox: **"Native: full access to your
   computer"**. You must tick it to continue.

| The sheet says | It allows |
|---|---|
| Read the selected items | Read what was selected when the plugin's action started |
| Read the current document | Read the whole current document |
| Read all open documents | Read every open document |
| Change the selected items / the current document / all open documents | Edit them. Changing includes reading. |
| Open files and folders you pick | Ask you to pick a file or folder, then read only that (PhotoCraft) |
| Save to files and folders you pick | Ask you to pick a folder or file, then write only there (PhotoCraft) |
| Open its own windows | Open floating windows. Panels need no permission. |
| Be told when things happen in the app (for example the selection changes) | Receive app events |

A permission is granted to the plugin in this app only. Setting a plugin up in EffectCraft does not
set it up in PhotoCraft.

**What native consent means.** A native plugin's helper program runs with your user account. It can
read and change your files, use the network and run other programs. Permissions do not limit it.
The app can stop the program, and Undo removes its changes to the document, but nothing can undo
what it did outside the document. Consent applies to one version: every update asks again.

From a terminal:

```sh
effectcraft-cli plugin permissions io.github.arnav-hiwarkar.cross-review
effectcraft-cli plugin setup io.github.arnav-hiwarkar.cross-review --all --grant document.edit:document
```

`setup` refuses and lists each missing permission until you grant them. `--all` turns on only the
parts this app can run. Add `--full-access` for a native plugin.

## 5. Using plugins

| | EffectCraft | PhotoCraft | FigCraft |
|---|---|---|---|
| Effects | Effects & Presets, and Effect ▸ category (default **Plugins**). An ordinary effect you can animate. | Filter ▸ Plugins ▸ category. Applies to the active layer of an RGB document. | Not available |
| Commands | Plugins ▸ plugin ▸ command | Plugins ▸ plugin ▸ command | Plugins ▸ plugin ▸ command |
| Panels | Plugins ▸ plugin ▸ panel (a floating panel) | Plugins ▸ plugin ▸ panel toggles it | Plugins menu toggles it; docked, with **Hide panel** and **Move to left/right sidebar** |
| Tools | Select from the Plugins menu; drag in the composition viewer | Select from the Plugins menu; drag on the canvas | Plugin tool strip above the toolbar, or the Plugins menu |
| Presets | Each preset is a command under Plugins ▸ plugin | In the plugin filter's dialog | — |
| LUTs | Listed for Apply Color LUT (`plugin.resources`) | Color Lookup ▸ From plugins | — |

While a plugin tool is selected you can still zoom and pan. Press **Escape** to cancel a gesture.

## 6. Undo

Each plugin action is **one undo step**, named `{action} · {plugin}`. A whole tool drag, from press
to release, is one step.

If a plugin action fails, takes too long, or loses its permission while it runs, the app undoes
everything that action changed.

While a plugin tool drag is in progress, the app refuses Undo and other edits until you finish or
cancel the drag.

Undo cannot reverse:

- files a plugin exported or saved (PhotoCraft). If a later file fails, earlier files stay, and the
  error says how many;
- anything a native plugin did outside the document.

## 7. Turning plugins off and revoking access

In the **Installed** tab, for each plugin:

- Untick a contribution to disable it.
- Press **Revoke** next to a granted permission.
- Untick **"Native: full access to your computer"** to withdraw native consent.
- **Remove** uninstalls it.

```sh
effectcraft-cli plugin disable <id> <contribution>
effectcraft-cli plugin revoke <id> <permission>
effectcraft-cli plugin remove <id>
```

These take effect without a restart, even if another app or the CLI made the change. Work in progress stops
and its changes are undone. A native helper program is stopped. In EffectCraft, effects from a
plugin that is turned off stay in your project and render unchanged, with a badge; final renders
are refused until the plugin is back.

## 8. Updates, pins and rollback

Nothing updates on its own. In the **Updates** tab, press **Check for updates**. An update that
needs more access shows **"Warning: wider permissions or native access; review setup"**. It waits
until you accept it in setup; the version you have keeps working until then.

In the **Installed** tab you can choose a version, then press **Set up selected version /
rollback**, **Pin selected version** or **Unpin**. Rolling back to a version whose saved data is
older warns: "Older version may need older saved data. Keep newer project data as placeholders."

```sh
effectcraft-cli plugin updates
effectcraft-cli plugin update <id>
effectcraft-cli plugin rollback <id> 1.0.0
effectcraft-cli plugin pin <id> "^1.0"
effectcraft-cli plugin unpin <id>
```

## 9. Problems and Resume

The **Problems** tab lists plugins that do not fit this app, plugins named in the kill list with the
reason, and plugins that failed to load.

If a plugin fails three times in a row, the app pauses it for the rest of the session:

- EffectCraft shows "{plugin}: paused after repeated failures" with **Resume** at the top of
  Settings ▸ Plugins.
- PhotoCraft shows **Paused** at the top of the Plugins window ("These plugins failed three times
  in a row and are paused until PhotoCraft restarts.") with **Resume**.
- FigCraft shows **Problems this session** in the Problems tab with **Resume**.

Resume starts the plugin again from a clean state. A paused plugin is not paused after a restart.

## 10. Safe mode

If an app misbehaves because of a plugin, start it with plugin code turned off:

```sh
EFFECTCRAFT_SAFE_MODE=1 effectcraft     # or: effectcraft --safe-mode
PHOTOCRAFT_SAFE_MODE=1 photocraft
FIGCRAFT_SAFE_MODE=1 figcraft
```

No plugin code runs. The plugin manager still works, so you can disable or remove the plugin.
EffectCraft and FigCraft show a safe mode notice in the manager. Restart normally to load plugins
again.

## 11. Projects that use plugins

EffectCraft and FigCraft remember which plugins a project uses. PhotoCraft does not yet.

When you open a project whose plugins are not ready, the app asks **"This project uses N
plugins"**. For each plugin it shows the status, with **Download** (not installed, or a different
version is needed), **Enable** (installed but not set up) and **Skip**. Then choose **Open without
them** or **Continue**. Opening a project never downloads or enables anything by itself.

Missing plugins keep their placeholders and saved data. In EffectCraft, missing effects render
unchanged and final renders are refused until the plugins are back. Missing panels or commands never
block export.

```sh
effectcraft-cli plugin project film.ecproj
figcraft-cli plugin project board.figcraft
```

## 12. Reporting a plugin

Press **Report plugin** in the Installed tab, or run `effectcraft-cli plugin report <id>`. Both
open or print a pre-filled issue in the catalogue repository with the plugin id and version.

## 13. The kill list

The signed catalogue carries a list of withdrawn releases. The app reads it when it refreshes the
catalogue (about every six hours, or when you press **Refresh catalogue** or run `plugin refresh`).
An entry can match a plugin id, versions, an exact file, or a vendor, so it can also match a
plugin you installed from a file.

| Level | Effect |
|---|---|
| Advisory | A warning in the Problems tab. Nothing is blocked. |
| Block new | No new installs, enables or updates of the matching releases. |
| Disable | The matching releases stop in every app, and stay off while the entry is listed. |

## 14. Command summary

| Task | Command |
|---|---|
| Search | `plugin search <words>` |
| Details | `plugin info <id>` |
| Install | `plugin install <id>` or `plugin install-file <file>` |
| Set up | `plugin setup <id> --enable <parts> --grant <permission> [--full-access]` |
| Permissions | `plugin permissions <id>`, `plugin grant <id> <permission>`, `plugin revoke <id> <permission>` |
| Turn a part on or off | `plugin enable <id> <part>`, `plugin disable <id> <part>` |
| Updates | `plugin updates`, `plugin update <id>`, `plugin rollback <id> <version>`, `plugin pin <id> <req>`, `plugin unpin <id>` |
| Remove | `plugin remove <id>` |
| Report | `plugin report <id>` |
| Check a project | `plugin project <file>` |

Prefix each with your app's CLI (`effectcraft-cli`, `photocraft-cli`, `figcraft-cli`). The full
list is in the [CLI reference](cli-reference.md).
