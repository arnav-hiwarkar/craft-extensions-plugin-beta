use craft::theme::Variant;
use craft::theme::variant::{Emphasis, Tone as FwTone};
use craft::ui::runtime::env::Label;
use craft_ext_manager::{Look, Tone, Word};

pub fn label(word: Word) -> Label {
    Label::Word(craft::i18n::Word::keyed(word.key(), word.english()))
}

pub fn tone(tone: Tone) -> FwTone {
    match tone {
        Tone::Neutral => FwTone::Neutral,
        Tone::Accent => FwTone::Accent,
        Tone::Info => FwTone::Info,
        Tone::Good => FwTone::Good,
        Tone::Warn => FwTone::Warn,
        Tone::Bad => FwTone::Bad,
    }
}

pub fn badge_variant(t: Tone) -> Variant {
    Variant::look(Emphasis::Soft, tone(t))
}

pub fn button_variant(look: Look) -> Variant {
    match look {
        Look::Primary => Variant::PRIMARY,
        Look::Secondary | Look::Busy => Variant::SECONDARY,
    }
}

pub fn notice_tone(error: bool) -> FwTone {
    if error { FwTone::Bad } else { FwTone::Good }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use craft_ext_manager::{Badge, Primary, WORDS};

    #[test]
    fn every_manager_word_reaches_the_framework_by_its_key_with_its_english() {
        for (word, key, english) in WORDS {
            match label(*word) {
                Label::Word(w) => assert_eq!((w.id, w.english), (*key, *english)),
                other => panic!("{key} became {other:?}"),
            }
        }
    }

    #[test]
    fn badges_and_buttons_take_the_tone_and_variant_their_meaning_names() {
        assert_eq!(badge_variant(Badge::NativeFull.tone()), Variant::look(Emphasis::Soft, FwTone::Bad));
        assert_eq!(badge_variant(Badge::Featured.tone()), Variant::look(Emphasis::Soft, FwTone::Info));
        assert_eq!(button_variant(Primary::Install.look()), Variant::PRIMARY);
        assert_eq!(button_variant(Primary::Open.look()), Variant::SECONDARY);
        assert_eq!(button_variant(Primary::Working.look()), Variant::SECONDARY);
        assert_eq!((notice_tone(true), notice_tone(false)), (FwTone::Bad, FwTone::Good));
    }
}
