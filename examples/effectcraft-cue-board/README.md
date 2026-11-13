# Cue board

EffectCraft-only example: composition marker table and layer notes. MIT licensed.

Run `craft-ext build --release`, `craft-ext check`, `craft-ext test`, then
`craft-ext dev --app effectcraft` and enable main with the declared permissions.
Open Plugins → Cue board → Cue board.

The Cues tab lists composition markers. Add cue writes at the playhead and is
one undo step; Rename top layer edits the first layer's name. The Guide tab
demonstrates a host-drawn canvas. The panel queries only project data and asks
for document edit permission; it has no filesystem or network access.

Panel/tool behavior must also be exercised in EffectCraft; the author tool
currently runs deterministic automated tests for effects only.
