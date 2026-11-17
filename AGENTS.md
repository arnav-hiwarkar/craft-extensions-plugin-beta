# Plugin platform contributor instructions

## Suite plugin replacement policy

When changing plugin implementations, app UI integration, lifecycle flows or merging
upstream plugin/UI changes in this private beta fork, apply the user-directed
[suite replacement rule](https://github.com/arnav-hiwarkar/craft-extensions-plugin-beta/blob/plugin-beta/docs/plugins/suite-replacement-rule.md). In the sibling beta workspace the same file is
`docs/plugins/suite-replacement-rule.md`. Inspect and learn from
each app's existing capabilities, migrate and verify the flows, then remove the
superseded implementations. Keep one shared platform and one canonical adapter per
app, coordinated with the current framework/UI work.
