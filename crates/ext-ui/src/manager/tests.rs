#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use super::*;
use craft_ext_core::{contract::Provided, id::AppId};
use craft_ext_manager::{Action, AppTarget, Tab, Word};
use std::time::{Duration, Instant};

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

fn frame(ctx: &egui::Context, draw: impl FnMut(&mut egui::Ui)) {
    let mut draw = draw;
    ctx.run_ui(egui::RawInput::default(), |ui| {
        egui::CentralPanel::default().show(ui, |ui| draw(ui));
    })
    .drop_without_applying_deltas();
}

#[test]
fn app_problems_draw_only_on_the_problems_tab_and_count_as_problems() {
    let root = temp_root("app-problems");
    let mut page = PluginManager::new(config(&root, &["effectcraft"]));
    let deadline = Instant::now() + Duration::from_secs(20);
    while !page.model().loaded() && Instant::now() < deadline {
        page.model_mut().poll();
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(page.model().loaded());
    let ctx = egui::Context::default();
    let mut calls = 0;
    for tab in [Tab::Discover, Tab::Problems, Tab::Installed, Tab::Problems, Tab::Updates] {
        page.model_mut().apply(Action::SetTab(tab));
        frame(&ctx, |ui| {
            page.show_with_app_problems(ui, &english, |ui| {
                calls += 1;
                ui.label("Resume a paused plugin");
                true
            })
        });
    }
    assert_eq!(calls, 2);
    assert_eq!(page.model().tab(), Tab::Updates);
    assert_eq!(page.model().rows(), []);
    std::fs::remove_dir_all(root).unwrap();
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
