#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use super::*;
use craft_ext_core::{contract::Provided, id::AppId};
use craft_ext_manager::{AppTarget, Word};

fn temp_root(name: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = std::env::temp_dir().join(format!("craft-ext-ui-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(root.join("remote")).unwrap();
    root
}

fn config(root: &std::path::Path, apps: &[&str]) -> Config {
    let targets = apps.iter().map(|a| AppTarget { app: AppId::new(a).unwrap(), provided: Provided::new(&[("effectcraft.session", 1, 1)]) }).collect();
    let mut config = Config::new(root.join("store"), targets, "linux-x86_64");
    config.catalogue_base = format!("file://{}/", root.join("remote").display());
    config
}

fn english(word: Word) -> String {
    word.english().to_string()
}

#[test]
fn a_prompt_with_nothing_to_fix_stays_open_until_the_user_answers() {
    let root = temp_root("prompt");
    let mut prompt = ProjectPrompt::new(config(&root, &["effectcraft"]), vec![]);
    let ctx = egui::Context::default();
    for _ in 0..3 {
        let mut outcome = Some(PromptOutcome { skipped: vec![], open_without: true });
        ctx.run_ui(egui::RawInput::default(), |ui| outcome = prompt.show(ui.ctx(), &english)).drop_without_applying_deltas();
        assert_eq!(outcome, None);
    }
    std::fs::remove_dir_all(root).unwrap();
}
