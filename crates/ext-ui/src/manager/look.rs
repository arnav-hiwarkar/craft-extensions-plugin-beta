use craft_ext_core::id::AppId;
use craft_ext_manager::{Look, Notice, Tone, Word};
use egui::{Color32, Visuals};

pub const TOAST_SECONDS: f64 = 4.0;
pub const ERROR_TOAST_SECONDS: f64 = 8.0;
pub const MAX_TOASTS: usize = 4;

pub trait Words {
    fn word(&self, word: Word) -> String;

    fn app(&self, app: &AppId) -> String {
        app.to_string()
    }
}

impl<F: Fn(Word) -> String> Words for F {
    fn word(&self, word: Word) -> String {
        self(word)
    }
}

pub fn tone_color(tone: Tone, visuals: &Visuals) -> Color32 {
    match tone {
        Tone::Neutral => visuals.weak_text_color(),
        Tone::Accent => visuals.selection.bg_fill,
        Tone::Info => visuals.hyperlink_color,
        Tone::Good => visuals.selection.stroke.color,
        Tone::Warn => visuals.warn_fg_color,
        Tone::Bad => visuals.error_fg_color,
    }
}

pub fn tone_fill(tone: Tone, visuals: &Visuals) -> Color32 {
    tone_color(tone, visuals).gamma_multiply(if visuals.dark_mode { 0.18 } else { 0.12 })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Filled { fill: Color32, text: Color32 },
    Plain,
    Spinner,
}

pub fn button(look: Look, visuals: &Visuals) -> Button {
    match look {
        Look::Primary => Button::Filled { fill: visuals.selection.bg_fill, text: visuals.selection.stroke.color },
        Look::Secondary => Button::Plain,
        Look::Busy => Button::Spinner,
    }
}

pub fn notice_tone(notice: &Notice) -> Tone {
    if notice.error { Tone::Bad } else { Tone::Good }
}

pub fn notice_text(notice: &Notice, words: &dyn Words) -> String {
    let mut text = words.word(notice.word);
    if !notice.subject.is_empty() {
        text.push_str(" · ");
        text.push_str(&notice.subject);
    }
    if !notice.detail.is_empty() {
        text.push_str(": ");
        text.push_str(&notice.detail);
    }
    text
}

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub text: String,
    pub tone: Tone,
    pub until: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Toasts {
    items: Vec<Toast>,
}

impl Toasts {
    pub fn push(&mut self, notice: &Notice, words: &dyn Words, now: f64) {
        let seconds = if notice.error { ERROR_TOAST_SECONDS } else { TOAST_SECONDS };
        self.items.push(Toast { text: notice_text(notice, words), tone: notice_tone(notice), until: now + seconds });
        let extra = self.items.len().saturating_sub(MAX_TOASTS);
        self.items.drain(..extra);
    }

    pub fn live(&mut self, now: f64) -> &[Toast] {
        self.items.retain(|t| t.until > now);
        &self.items
    }

    pub fn next_expiry(&self, now: f64) -> Option<f64> {
        self.items.iter().map(|t| t.until - now).fold(None, |best, left| Some(best.map_or(left, |b: f64| b.min(left))))
    }
}

pub fn initial(name: &str) -> String {
    name.chars().find(|c| c.is_alphanumeric()).map(|c| c.to_uppercase().collect()).unwrap_or_else(|| "?".into())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    const TONES: [Tone; 6] = [Tone::Neutral, Tone::Accent, Tone::Info, Tone::Good, Tone::Warn, Tone::Bad];

    fn english(word: Word) -> String {
        word.english().to_string()
    }

    fn notice(word: Word, subject: &str, detail: &str, error: bool) -> Notice {
        Notice { word, plugin: None, subject: subject.into(), detail: detail.into(), error }
    }

    #[test]
    fn every_tone_takes_a_distinct_colour_from_the_dark_theme() {
        let dark = Visuals::dark();
        assert_eq!(
            TONES.map(|t| tone_color(t, &dark)),
            [
                Color32::from_rgba_premultiplied(84, 84, 84, 153),
                Color32::from_rgb(0, 92, 128),
                Color32::from_rgb(90, 170, 255),
                Color32::from_rgb(192, 222, 255),
                Color32::from_rgb(255, 143, 0),
                Color32::from_rgb(255, 0, 0),
            ]
        );
    }

    #[test]
    fn every_tone_takes_a_distinct_colour_from_the_light_theme() {
        let light = Visuals::light();
        assert_eq!(
            TONES.map(|t| tone_color(t, &light)),
            [
                Color32::from_rgba_premultiplied(48, 48, 48, 153),
                Color32::from_rgb(144, 209, 255),
                Color32::from_rgb(0, 155, 255),
                Color32::from_rgb(0, 83, 125),
                Color32::from_rgb(255, 100, 0),
                Color32::from_rgb(255, 0, 0),
            ]
        );
    }

    #[test]
    fn a_theme_change_changes_the_colours() {
        let mut custom = Visuals::dark();
        custom.warn_fg_color = Color32::from_rgb(200, 180, 20);
        custom.selection.bg_fill = Color32::from_rgb(10, 120, 60);
        assert_eq!(tone_color(Tone::Warn, &custom), Color32::from_rgb(200, 180, 20));
        assert_eq!(tone_color(Tone::Accent, &custom), Color32::from_rgb(10, 120, 60));
        assert_eq!(tone_fill(Tone::Bad, &custom), Color32::from_rgb(255, 0, 0).gamma_multiply(0.18));
    }

    #[test]
    fn install_update_and_set_up_are_filled_open_is_plain_and_working_spins() {
        let dark = Visuals::dark();
        let filled = Button::Filled { fill: Color32::from_rgb(0, 92, 128), text: Color32::from_rgb(192, 222, 255) };
        assert_eq!([Look::Primary, Look::Secondary, Look::Busy].map(|l| button(l, &dark)), [filled, Button::Plain, Button::Spinner]);
        let light = Visuals::light();
        assert_eq!(button(Look::Primary, &light), Button::Filled { fill: Color32::from_rgb(144, 209, 255), text: Color32::from_rgb(0, 83, 125) });
    }

    #[test]
    fn a_notice_reads_as_word_subject_and_detail_in_the_host_language() {
        let shout = |w: Word| w.english().to_uppercase();
        assert_eq!(notice_text(&notice(Word::NoticeInstalled, "io.github.alice.grain", "1.2.0", false), &shout), "INSTALLED · io.github.alice.grain: 1.2.0");
        assert_eq!(notice_text(&notice(Word::NoticeRefreshed, "", "", false), &english), "Catalogue refreshed");
        assert_eq!(notice_text(&notice(Word::NoticeFailed, "", "offline", true), &english), "Could not complete: offline");
    }

    #[test]
    fn toasts_expire_errors_stay_longer_and_only_the_newest_four_show() {
        let mut toasts = Toasts::default();
        toasts.push(&notice(Word::NoticeInstalled, "a", "", false), &english, 10.0);
        toasts.push(&notice(Word::NoticeFailed, "b", "", true), &english, 10.0);
        assert_eq!(toasts.next_expiry(10.0), Some(4.0));
        assert_eq!(toasts.live(13.9).iter().map(|t| (t.text.as_str(), t.tone)).collect::<Vec<_>>(), [("Installed · a", Tone::Good), ("Could not complete · b", Tone::Bad)]);
        assert_eq!(toasts.live(14.0).iter().map(|t| t.text.as_str()).collect::<Vec<_>>(), ["Could not complete · b"]);
        assert_eq!(toasts.live(18.0), []);
        for name in ["1", "2", "3", "4", "5"] {
            toasts.push(&notice(Word::NoticeSetUp, name, "", false), &english, 20.0);
        }
        assert_eq!(toasts.live(20.0).iter().map(|t| t.text.as_str()).collect::<Vec<_>>(), ["Set up · 2", "Set up · 3", "Set up · 4", "Set up · 5"]);
        assert_eq!(toasts.next_expiry(20.0), Some(4.0));
    }

    #[test]
    fn an_icon_without_a_picture_shows_the_first_letter() {
        assert_eq!([initial("grain"), initial("  über"), initial("3D tools"), initial("—")], ["G", "Ü", "3", "?"]);
    }

    #[test]
    fn an_app_name_defaults_to_its_id() {
        assert_eq!(english.app(&AppId::new("effectcraft").unwrap()), "effectcraft");
    }
}
