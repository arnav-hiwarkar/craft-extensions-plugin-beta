# Maintain the plugin catalogue

Use this procedure for listing admission, metadata publication and incident response.
The catalogue contains metadata; authors host package bytes on their own GitHub
Releases. The detailed repository policy is in
[craft-plugins/MAINTAINERS.md](https://github.com/arnav-hiwarkar/craft-plugins/blob/main/MAINTAINERS.md).
The [store reference](store-and-distribution.md) defines records and trust rules.
The [publication map](maps.md#10-catalogue-publication-and-revocation) shows the flow.

## Admit a vendor and listing

1. Establish the vendor's GitHub login, numeric account ID and repository ownership.
   Apply namespace and verified-vendor rules from the catalogue policy.
2. Confirm public source, the accepted OSI license expression and the free/non-monetized
   listing policy. Record any policy-required human review notes.
3. Check the listing ID, vendor, repository identity and declared apps/contracts.
   Do not advertise FigCraft raster.filter or deferred theme activation.
4. Validate the release record's version, tag, commit, archive size, SHA-256, manifest
   and asset URL. The asset must belong to the declared repository and release tag.
5. Run `catalogue check` using the exact syntax in the [CLI reference](cli-reference.md).
   Review its manual checklist. One maintainer allows a listing; no review bot is required.

A vendor tier identifies the publisher. It does not grant app permissions, certify
all runtime behavior or make native code sandboxed. Store admission and user consent
are separate decisions. App account sign-in and reviews are not implemented.

## Build and publish metadata

1. Confirm the authorized publication target and repository before writing to it.
   Private beta work stays on the user's private forks/mirrors.
2. Build package metadata and the next serial index with the catalogue tools. Review
   the output, freshness timestamps, revocations and release digests.
3. Sign the exact generated index with the authorized private maintainer key. Keep
   that key outside repositories, logs and packages. Never transfer it as a testing shortcut.
4. Publish the matching package metadata, index and signature together. Keep archives
   on authors' Releases; do not replace package bytes under an existing version/digest.
5. Refresh from a disposable consumer store and verify signature, catalogue freshness,
   package metadata hash, archive digest and the intended app compatibility.

The index expires after 30 days under the current builder. Refresh and re-sign in
advance. Key rotation requires the corresponding trusted public-key change in
consumers; it is not a local signature-bypass option.

## Handle reports and revocations

1. Direct users to the package/version report URL and GitHub issue form.
2. Triage manually under the published report policy. Review evidence, duplicate
   reports, affected digests and vendor identity before deciding severity.
3. Add an advisory, block-new or disable entry as appropriate. Scope it to the
   affected package/version, digest or vendor and explain the reason.
4. Build, sign and publish the next catalogue snapshot.
5. Verify the intended result after consumer refresh: advisory warns; block-new
   prevents new affected activation/install choices; disable stops matching work.

Do not promise immediate propagation before consumers refresh. Revocation cannot
undo native external effects or files already published by a plugin.

## Test without production secrets

Use a complete signed test-catalogue artifact and its public TEST-KEY.pub, or create
an explicitly authorized disposable test catalogue with its own throwaway key.
The debug test-key override is limited to file URLs with file access allowed. It
never authorizes HTTPS or release consumers and does not skip digest/freshness checks.
See [isolated Mac setup](installation-and-testing.md).

Current build/smoke evidence and known gaps are in [STATUS](../../STATUS.md) and
[TESTING](../../TESTING.md). Those records supersede the earlier source-only handoff.
A docs update does not create new runtime evidence. CI, package publication and
release actions follow the authorization applicable to that phase; this documentation
pass performs no new build or catalogue signing.
