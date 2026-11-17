macro_rules! words {
    ($($word:ident $key:literal $english:literal,)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum Word {
            $($word,)*
        }

        pub const WORDS: &[(Word, &str, &str)] = &[$((Word::$word, $key, $english),)*];

        impl Word {
            pub fn key(self) -> &'static str {
                match self {
                    $(Word::$word => $key,)*
                }
            }

            pub fn english(self) -> &'static str {
                match self {
                    $(Word::$word => $english,)*
                }
            }
        }
    };
}

words! {
    TabDiscover "plugins.manager.tab.discover" "Discover",
    TabInstalled "plugins.manager.tab.installed" "Installed",
    TabUpdates "plugins.manager.tab.updates" "Updates",
    TabProblems "plugins.manager.tab.problems" "Problems",
    SearchPlaceholder "plugins.manager.search.placeholder" "Search plugins",
    FilterApp "plugins.manager.filter.app" "App",
    FilterCategory "plugins.manager.filter.category" "Category",
    FilterTier "plugins.manager.filter.tier" "Trust",
    FilterKind "plugins.manager.filter.kind" "Kind",
    FilterInstalled "plugins.manager.filter.installed" "Installed",
    FilterHasUpdate "plugins.manager.filter.has-update" "Has update",
    SortRelevance "plugins.manager.sort.relevance" "Relevance",
    SortPopular "plugins.manager.sort.popular" "Popular",
    SortRecentlyUpdated "plugins.manager.sort.recently-updated" "Recently updated",
    SortName "plugins.manager.sort.name" "Name",
    ActionInstall "plugins.manager.action.install" "Install",
    ActionUpdate "plugins.manager.action.update" "Update",
    ActionUpdateAll "plugins.manager.action.update-all" "Update all",
    ActionUninstall "plugins.manager.action.uninstall" "Uninstall",
    ActionEnable "plugins.manager.action.enable" "Turn on",
    ActionDisable "plugins.manager.action.disable" "Turn off",
    ActionGrant "plugins.manager.action.grant" "Allow",
    ActionRevoke "plugins.manager.action.revoke" "Remove access",
    ActionPin "plugins.manager.action.pin" "Keep this version",
    ActionUnpin "plugins.manager.action.unpin" "Allow updates",
    ActionRollback "plugins.manager.action.rollback" "Use this version",
    ActionTurnBackOn "plugins.manager.action.turn-back-on" "Turn back on",
    ActionInstallMissing "plugins.manager.action.install-missing" "Install missing plugins",
    ActionSetUp "plugins.manager.action.set-up" "Set up",
    ActionConfirmSetup "plugins.manager.action.confirm-setup" "Confirm setup",
    ActionCancel "plugins.manager.action.cancel" "Cancel",
    ActionRefresh "plugins.manager.action.refresh" "Refresh catalogue",
    ActionReport "plugins.manager.action.report" "Report plugin",
    StateInstalled "plugins.manager.state.installed" "Installed",
    StateWorking "plugins.manager.state.working" "Working…",
    StateDownloading "plugins.manager.state.downloading" "Downloading",
    StateApplying "plugins.manager.state.applying" "Saving your choices",
    StateChecking "plugins.manager.state.checking" "Checking the project",
    StateLoading "plugins.manager.state.loading" "Loading",
    TierOfficial "plugins.manager.tier.official" "Official",
    TierVerified "plugins.manager.tier.verified" "Verified vendor",
    TierCommunity "plugins.manager.tier.community" "Community",
    TierLocal "plugins.manager.tier.local" "Not reviewed (local file)",
    TierDev "plugins.manager.tier.dev" "Development",
    BadgeBuiltFromSource "plugins.manager.badge.built-from-source" "Built from source",
    BadgeNativeFull "plugins.manager.badge.native-full" "Native: full access",
    BadgeNativeRestricted "plugins.manager.badge.native-restricted" "Native: restricted",
    BadgeAdvisory "plugins.manager.badge.advisory" "Advisory",
    BadgeFeatured "plugins.manager.badge.featured" "Featured",
    WarningNativeFull "plugins.manager.warning.native-full" "This plugin runs its own program with full access to your computer, outside the plugin sandbox.",
    WarningNativeRestricted "plugins.manager.warning.native-restricted" "This plugin runs its own program with no network and no access to your files outside its own folder.",
    ConsentFullAccess "plugins.manager.consent.full-access" "Allow full access to this computer",
    SetupChooseContribution "plugins.manager.setup.choose-contribution" "Choose at least one part of the plugin to turn on.",
    SetupGrantRequired "plugins.manager.setup.grant-required" "Allow the access marked required, or turn off the parts that need it.",
    SetupConsentNative "plugins.manager.setup.consent-native" "Allow full access to continue.",
    SetupNoHelper "plugins.manager.setup.no-helper" "This plugin has no program for this computer.",
    SetupIncompatible "plugins.manager.setup.incompatible" "This version does not work in this app.",
    PermissionRequired "plugins.manager.permission.required" "Required",
    PermissionOptional "plugins.manager.permission.optional" "Optional",
    ProblemQuarantined "plugins.manager.problem.quarantined" "Turned off after a crash",
    ProblemKillListed "plugins.manager.problem.kill-listed" "Removed for safety",
    ProblemNoHelper "plugins.manager.problem.no-helper" "No program for this computer",
    ProblemIncompatible "plugins.manager.problem.incompatible" "Does not work with this app version",
    ProblemConsentPending "plugins.manager.problem.consent-pending" "Waiting for your approval",
    ProblemNativeConsentPending "plugins.manager.problem.native-consent-pending" "Waiting for your approval of full access",
    ProblemUncleanExit "plugins.manager.problem.unclean-exit" "Was running when the app last quit unexpectedly",
    NoticeInstalled "plugins.manager.notice.installed" "Installed",
    NoticeUpdated "plugins.manager.notice.updated" "Updated",
    NoticeRolledBack "plugins.manager.notice.rolled-back" "Switched version",
    NoticeSetUp "plugins.manager.notice.set-up" "Set up",
    NoticeUninstalled "plugins.manager.notice.uninstalled" "Uninstalled",
    NoticeChanged "plugins.manager.notice.changed" "Saved",
    NoticeRefreshed "plugins.manager.notice.refreshed" "Catalogue refreshed",
    NoticeAlreadyReady "plugins.manager.notice.already-ready" "Already installed",
    NoticeUnavailable "plugins.manager.notice.unavailable" "Not available",
    NoticeFailed "plugins.manager.notice.failed" "Could not complete",
    EmptyDiscover "plugins.manager.empty.discover" "No plugins match your search.",
    EmptyInstalled "plugins.manager.empty.installed" "No plugins installed.",
    EmptyUpdates "plugins.manager.empty.updates" "Everything is up to date.",
    EmptyProblems "plugins.manager.empty.problems" "No problems.",
    Disclaimer "plugins.manager.disclaimer" "Plugins are made by their authors. ArtCraft lists them but does not build, host or guarantee them.",
}
