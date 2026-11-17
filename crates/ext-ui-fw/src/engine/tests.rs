#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use super::*;
use craft_ext_core::form::Word;
use craft_ext_core::ui::tree::{
    Badge as TreeBadge, Button as TreeButton, Canvas, Check, Collapsible, ColorInput, Column, Grid as TreeGrid, Group, Icon as TreeIcon, Image, Item,
    Label as TreeLabel, List as TreeList, Notice as TreeNotice, NumberInput, Progress as TreeProgress, Radio as TreeRadio, Scroll as TreeScroll,
    Select as TreeSelect, Slider, Spacer, Stack as TreeStack, Tab as TreeTab, Table as TreeTable, Tabs, TextArea, TextInput, Unknown,
};

fn button(id: &str, icon: Option<&str>, icon_only: bool, variant: Variant) -> Node {
    Node::Button(TreeButton { id: id.into(), label: id.into(), icon: icon.map(Into::into), icon_only, variant, hint: None, disabled: false })
}

fn check(id: &str) -> Check {
    Check { id: id.into(), label: id.into(), value: false, hint: None, disabled: false }
}

fn text_input(search: bool) -> Node {
    Node::TextInput(TextInput {
        id: "find".into(),
        label: "Find".into(),
        value: String::new(),
        placeholder: None,
        hint: None,
        disabled: false,
        search,
    })
}

fn list(empty: Option<&str>) -> Node {
    Node::List(TreeList {
        id: "presets".into(),
        label: "Presets".into(),
        items: vec![Item { key: "a".into(), label: "A".into(), detail: None, icon: None }],
        selection: Selection::Single,
        selected: vec![],
        max_height: None,
        hint: None,
        empty: empty.map(Into::into),
    })
}

fn progress(value: Option<f32>) -> Node {
    Node::Progress(TreeProgress { value, label: None })
}

fn level_1_2() -> Vec<Node> {
    vec![
        Node::Row(TreeStack { children: vec![] }),
        Node::Grid(TreeGrid { columns: 2, children: vec![], striped: false }),
        Node::Group(Group { label: "G".into(), children: vec![] }),
        Node::Collapsible(Collapsible { id: "c".into(), label: "C".into(), open: true, children: vec![] }),
        Node::Scroll(TreeScroll::default()),
        Node::Label(TreeLabel { text: "Exposure".into(), style: TextStyle::Heading, tone: Tone::Neutral, icon: None }),
        Node::Separator,
        Node::Spacer(Spacer { size: 8.0 }),
        button("apply", None, false, Variant::Primary),
        button("trash", Some("trash"), true, Variant::Danger),
        Node::Checkbox(check("k")),
        Node::Toggle(check("t")),
        Node::Slider(Slider { id: "s".into(), label: "S".into(), value: 1.0, min: 0.0, max: 2.0, step: None, hint: None, disabled: false }),
        Node::Number(NumberInput {
            id: "n".into(),
            label: "N".into(),
            value: 1.0,
            min: None,
            max: None,
            step: None,
            decimals: None,
            unit: None,
            hint: None,
            disabled: false,
        }),
        text_input(false),
        text_input(true),
        Node::TextArea(TextArea {
            id: "a".into(),
            label: "A".into(),
            value: String::new(),
            rows: None,
            placeholder: None,
            mono: false,
            hint: None,
            disabled: false,
        }),
        Node::Select(TreeSelect { id: "e".into(), label: "E".into(), options: vec!["x".into()], selected: 0, hint: None, disabled: false }),
        Node::Radio(TreeRadio {
            id: "r".into(),
            label: "R".into(),
            options: vec!["x".into()],
            selected: 0,
            horizontal: false,
            hint: None,
            disabled: false,
        }),
        Node::Color(ColorInput { id: "o".into(), label: "O".into(), rgba: [1.0, 0.0, 0.0, 1.0], hint: None, disabled: false }),
        Node::Tabs(Tabs {
            id: "tb".into(),
            tabs: vec![TreeTab { id: "one".into(), label: "One".into(), icon: None, children: vec![] }],
            selected: None,
        }),
        list(Some("No presets yet")),
        Node::Table(TreeTable {
            id: "ta".into(),
            label: "Ta".into(),
            columns: vec![Column { label: "Name".into(), width: None, align: Align::Left }],
            rows: vec![],
            selection: Selection::None,
            selected: vec![],
            max_height: None,
            hint: None,
            empty: None,
        }),
        progress(Some(0.5)),
        progress(None),
        Node::Icon(TreeIcon { name: "star".into(), size: None, label: None, rgba: None }),
        Node::Image(Image { width: 1, height: 1, rgba: vec![0, 0, 0, 255], name: None, hidden: false, size: None, alt: None }),
        Node::Badge(TreeBadge { text: "Beta".into(), tone: Tone::Info }),
        Node::Notice(TreeNotice { text: "Slow on large images".into(), title: Some("Heads up".into()), tone: Tone::Warn }),
        Node::Canvas(Canvas {
            id: "cv".into(),
            label: "Cv".into(),
            width: 10.0,
            height: 10.0,
            ops: vec![],
            listen: vec![],
            cursor: None,
            hint: None,
        }),
        Node::Unknown(Unknown { kind: "later-node".into(), fallback: None, rest: Default::default() }),
    ]
}

#[test]
fn every_node_of_level_1_2_draws_with_the_component_section_4_2_names() {
    let kinds: Vec<Drawn> = level_1_2().iter().map(drawn).collect();
    assert_eq!(
        kinds,
        vec![
            Drawn::Row,
            Drawn::Grid,
            Drawn::GroupBox,
            Drawn::Section,
            Drawn::Scroll,
            Drawn::Text,
            Drawn::Divider,
            Drawn::Gap,
            Drawn::Button,
            Drawn::IconButton,
            Drawn::Checkbox,
            Drawn::Switch,
            Drawn::SliderField,
            Drawn::NumberField,
            Drawn::TextField,
            Drawn::SearchField,
            Drawn::TextField,
            Drawn::Select,
            Drawn::RadioGroup,
            Drawn::Egui,
            Drawn::TabStrip,
            Drawn::List,
            Drawn::Table,
            Drawn::Progress,
            Drawn::Spinner,
            Drawn::Icon,
            Drawn::Picture,
            Drawn::Badge,
            Drawn::Notice,
            Drawn::Egui,
            Drawn::Fallback,
        ]
    );
    assert_eq!(drawn(&Node::Column(TreeStack::default())), Drawn::Stack);
    assert_eq!(drawn(&button("x", Some("trash"), false, Variant::Ghost)), Drawn::Button);
}

#[test]
fn tones_variants_and_text_styles_become_framework_meanings() {
    let tones: Vec<FwTone> = [Tone::Neutral, Tone::Accent, Tone::Info, Tone::Good, Tone::Warn, Tone::Bad].into_iter().map(tone).collect();
    assert_eq!(tones, vec![FwTone::Neutral, FwTone::Accent, FwTone::Info, FwTone::Good, FwTone::Warn, FwTone::Bad]);
    let variants: Vec<FwVariant> = [Variant::Primary, Variant::Secondary, Variant::Danger, Variant::Ghost].into_iter().map(variant).collect();
    assert_eq!(variants, vec![FwVariant::PRIMARY, FwVariant::SECONDARY, FwVariant::DANGER, FwVariant::GHOST]);
    assert_eq!(badge(Tone::Warn), FwVariant::look(Emphasis::Soft, FwTone::Warn));
    let roles: Vec<(TypeRole, Shade)> = [TextStyle::Body, TextStyle::Heading, TextStyle::Strong, TextStyle::Small, TextStyle::Weak, TextStyle::Mono]
        .into_iter()
        .map(type_role)
        .collect();
    assert_eq!(
        roles,
        vec![
            (TypeRole::Body, Shade::Normal),
            (TypeRole::Heading, Shade::Normal),
            (TypeRole::BodyStrong, Shade::Normal),
            (TypeRole::Caption, Shade::Normal),
            (TypeRole::Body, Shade::Muted),
            (TypeRole::Mono, Shade::Normal),
        ]
    );
    assert_eq!([Selection::None, Selection::Single, Selection::Multi].map(select_mode), [SelectMode::None, SelectMode::Single, SelectMode::Multi]);
    assert_eq!([Align::Left, Align::Center, Align::Right].map(column_align), [ColumnAlign::Start, ColumnAlign::Start, ColumnAlign::End]);
}

#[test]
fn controls_send_the_values_the_egui_engine_sends_and_only_for_a_change() {
    assert_eq!(number_edit(1.0, Some(2.5)), Some(UiValue::Number(2.5)));
    assert_eq!(number_edit(1.0, Some(1.0)), None);
    assert_eq!(number_edit(1.0, Some(f64::NAN)), None);
    assert_eq!(number_edit(1.0, None), None);
    assert_eq!(bool_edit(false, Some(true)), Some(UiValue::Bool(true)));
    assert_eq!(bool_edit(true, Some(true)), None);
    assert_eq!(text_edit("a", Some(&"ab".to_owned())), Some(UiValue::Text("ab".into())));
    assert_eq!(text_edit("a", Some(&"a".to_owned())), None);
    assert_eq!(index_edit(0, Some(2)), Some(UiValue::Index(2)));
    assert_eq!(index_edit(2, Some(2)), None);
    let shown = vec!["a".to_owned()];
    assert_eq!(selection_edit(&shown, SelectionChange::Set(vec!["b".into()])), Some(UiValue::Keys(vec!["b".into()])));
    assert_eq!(selection_edit(&shown, SelectionChange::Toggle("b".into())), Some(UiValue::Keys(vec!["a".into(), "b".into()])));
    assert_eq!(selection_edit(&shown, SelectionChange::Toggle("a".into())), Some(UiValue::Keys(vec![])));
    assert_eq!(selection_edit(&shown, SelectionChange::Clear), Some(UiValue::Keys(vec![])));
    assert_eq!(selection_edit(&shown, SelectionChange::Set(vec!["a".into()])), None);
    let options = [Choice { id: "fast".into(), label: Word::from("Fast") }, Choice { id: "best".into(), label: Word::from("Best") }];
    assert_eq!(choice_edit(&options, "fast", Some(1)), Some(UiValue::Text("best".into())));
    assert_eq!(choice_edit(&options, "fast", Some(0)), None);
    assert_eq!(choice_edit(&options, "fast", Some(7)), None);
}

#[test]
fn form_sections_split_the_rows_that_follow_them() {
    let row = |key: &str, kind: ParamKind| Param {
        key: key.into(),
        label: Word::from(key),
        kind,
        unit: None,
        visible_when: None,
        enabled_when: None,
        help: None,
    };
    let params = vec![
        row("a", ParamKind::Bool { default: true }),
        row("look", ParamKind::Section),
        row("b", ParamKind::Bool { default: true }),
        row("c", ParamKind::Bool { default: false }),
    ];
    let runs: Vec<(Option<&str>, Vec<&str>)> =
        sections(&params).into_iter().map(|(title, rows)| (title.map(|p| p.key.as_str()), rows.iter().map(|p| p.key.as_str()).collect())).collect();
    assert_eq!(runs, vec![(None, vec!["a"]), (Some("look"), vec!["b", "c"])]);
    let leading: Vec<usize> = sections(&params[1..]).into_iter().map(|(_, rows)| rows.len()).collect();
    assert_eq!(leading, vec![2]);
}
