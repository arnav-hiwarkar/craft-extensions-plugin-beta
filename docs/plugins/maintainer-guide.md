# Maintaining the beta catalogue

The catalogue repository contains metadata; authors host packages on their own
GitHub Releases. Follow `craft-plugins/MAINTAINERS.md` for vendor admission and
listing review. Use one maintainer to allow a listing; no automated review bot is
required. Preserve OSI/public-source checks, vendor identity and namespace checks,
release-tag/commit provenance and digest-pinned downloads. Vendor tier identifies
the publisher; it does not grant permissions or make native code sandboxed.

Review compatibility per contribution and app contract. Do not list FigCraft as
supporting raster.filter. Installation must not enable contributions or grant
permissions. App, launcher and CLI consumers use the shared Manager setup, version,
pin, rollback and revocation rules. Projects may prompt for missing packages but
must not install or enable them on open without the user's choice.

Before updating metadata, validate listing/license/source/release records with the
catalogue tools documented in that repository. Review report links and revocation
criteria there. Revocation can target a vendor, version or digest; all consumers
must refresh it and stop affected work. Keep serial/signature verification enabled.
Do not commit a signing key, private catalogue fixture or generated binaries.

The current Mac phase completes source and documentation only. Catalogue generation,
signing, upload, app/guest packaging, CI and releases have not been authorized for
this phase. The source review and deferred runtime/platform matrix are recorded in
[the evidence ledger](session-handoff/MAC-EVIDENCE.md). Historical local signed
catalogue runs do not prove real GitHub downloads or current binaries. After the
separate verification approval, validate a disposable store through app, launcher
and CLI installation, setup, grants, update, pin, rollback, revocation, safe mode,
project prompting and export refusal before considering release approval.
