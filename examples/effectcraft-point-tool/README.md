# Point keyframes

EffectCraft-only example: place position keyframes on selected layers. MIT licensed.

Run `craft-ext build --release`, `craft-ext check`, `craft-ext test`, then
`craft-ext dev --app effectcraft` and enable main with the declared permissions.
Open Plugins → Point keyframes → Point keyframes.

Select layers first, then select Point keyframes from Plugins. Click or
release a drag to add a position keyframe at the playhead. Guides track the
pointer in composition pixels. Escape rolls back the gesture and clears guides;
middle drag/Space and wheel navigation remain host-owned. The tool needs read
selection and document edit grants; it never imports or exports a file.

Panel/tool behavior must also be exercised in EffectCraft; the author tool
currently runs deterministic automated tests for effects only.
