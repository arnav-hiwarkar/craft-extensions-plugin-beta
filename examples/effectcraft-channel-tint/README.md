# Channel tint

EffectCraft-only example: channel tint with conditional controls. MIT licensed.

Run `craft-ext build --release`, `craft-ext check`, `craft-ext test`, then
`craft-ext dev --app effectcraft` and enable main with the declared permissions.
Open Plugins → Channel tint → Channel tint.

Apply Channel tint to a colored layer. Reduction attenuates red or blue.
Protection skips bright red pixels; Mix a tint exposes a color picker and mixes
straight tint color using the input alpha. All outputs remain premultiplied and
alpha is preserved. The effect is deterministic and requires no permissions.

Panel/tool behavior must also be exercised in EffectCraft; the author tool
currently runs deterministic automated tests for effects only.
