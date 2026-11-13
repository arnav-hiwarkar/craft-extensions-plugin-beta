//! Headless renderer tests: `egui::Context::run` with synthetic input, asserting the events the
//! renderer emits. Widgets are found through `Host::widget`, as an app's automation finds them.

use super::*;
use craft_ext_core::ui::tree::{PointerButton as PB, PointerEvent, PointerKind};
use egui::{Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, pos2, vec2};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Default)]
struct Rec {
    rects: BTreeMap<String, Rect>,
}

impl Host for Rec {
    fn widget(&mut self, _surface: &str, id: &str, rect: Rect, _label: &str) {
        self.rects.insert(id.to_string(), rect);
    }
}

/// A headless app: one panel drawing one surface.
struct Harness {
    ctx: egui::Context,
    ui: PluginUi,
    host: Rec,
    tree: Value,
    rev: u64,
    time: f64,
}

impl Harness {
    fn new(tree: Value) -> Self {
        let mut h = Self { ctx: egui::Context::default(), ui: PluginUi::default(), host: Rec::default(), tree, rev: 1, time: 0.0 };
        h.frame(vec![]);
        h.frame(vec![]);
        h
    }

    fn frame(&mut self, events: Vec<Event>) -> Vec<UiEvent> {
        self.frame_with(events, Modifiers::NONE)
    }

    fn frame_with(&mut self, mut events: Vec<Event>, modifiers: Modifiers) -> Vec<UiEvent> {
        self.time += 0.02;
        events.insert(0, Event::ModifiersChanged(modifiers));
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 1200.0))),
            time: Some(self.time),
            events,
            ..RawInput::default()
        };
        let mut out = Vec::new();
        let (ui, host, tree, rev) = (&mut self.ui, &mut self.host, &self.tree, self.rev);
        self.ctx
            .run_ui(input, |root| {
                egui::CentralPanel::default().show(root, |pane| {
                    out = ui.draw_json(pane, "panel", rev, Some(tree), host);
                });
            })
            .drop_without_applying_deltas();
        out
    }

    fn at(&self, id: &str) -> Pos2 {
        self.host.rects.get(id).unwrap_or_else(|| panic!("widget {id} was not drawn: {:?}", self.host.rects.keys())).center()
    }

    /// A lone click: far enough in time from any earlier click not to count as a double click.
    fn click_at(&mut self, p: Pos2, modifiers: Modifiers) -> Vec<UiEvent> {
        self.time += 1.0;
        self.press_release(p, modifiers)
    }

    fn double_click_at(&mut self, p: Pos2) -> Vec<UiEvent> {
        let mut out = self.click_at(p, Modifiers::NONE);
        out.extend(self.press_release(p, Modifiers::NONE));
        out
    }

    fn press_release(&mut self, p: Pos2, modifiers: Modifiers) -> Vec<UiEvent> {
        let mut out = self.frame_with(vec![Event::PointerMoved(p)], modifiers);
        out.extend(self.frame_with(vec![Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: true, modifiers }], modifiers));
        out.extend(self.frame_with(vec![Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: false, modifiers }], modifiers));
        out.extend(self.frame_with(vec![], modifiers));
        out
    }

    fn click(&mut self, id: &str) -> Vec<UiEvent> {
        let p = self.at(id);
        self.click_at(p, Modifiers::NONE)
    }
}

fn values(events: &[UiEvent]) -> Vec<(String, UiValue)> {
    events.iter().map(|e| (e.widget.clone(), e.value.clone())).collect()
}

fn pointer(e: &UiEvent) -> &PointerEvent {
    match &e.value {
        UiValue::Pointer(p) => p,
        other => panic!("not a pointer event: {other:?}"),
    }
}

#[test]
fn engine_choice_is_the_apps_and_framework_falls_back_to_egui() {
    assert_eq!(PluginUi::default().engine(), EngineChoice::Egui);
    let ui = PluginUi::new(EngineChoice::Framework);
    assert_eq!(ui.choice(), EngineChoice::Framework);
    assert_eq!(ui.engine(), EngineChoice::Egui);
}

#[test]
fn buttons_checkboxes_and_toggles_send_their_values_with_the_drawn_rev() {
    let mut h = Harness::new(json!({"type":"column","children":[
        {"type":"button","id":"go","label":"Go"},
        {"type":"button","id":"del","label":"Delete","icon":"trash","icon_only":true},
        {"type":"button","id":"off","label":"Off","disabled":true},
        {"type":"checkbox","id":"c","label":"Check","value":false},
        {"type":"toggle","id":"t","label":"Live","value":true}
    ]}));
    let e = h.click("go");
    assert_eq!(values(&e), vec![("go".into(), UiValue::Click)]);
    assert!(e.iter().all(|e| e.surface == "panel" && e.rev == 1));
    assert_eq!(values(&h.click("del")), vec![("del".into(), UiValue::Click)]);
    assert!(h.click("off").is_empty(), "a disabled button sends nothing");
    assert_eq!(values(&h.click("c")), vec![("c".into(), UiValue::Bool(true))]);
    assert_eq!(values(&h.click("t")), vec![("t".into(), UiValue::Bool(false))]);
}

#[test]
fn a_slider_sends_once_when_the_drag_ends_and_survives_a_redraw_mid_drag() {
    let mut h = Harness::new(json!({"type":"slider","id":"s","label":"Size","value":0.0,"min":0.0,"max":100.0}));
    let r = h.host.rects["s"];
    let start = pos2(r.left() + 20.0, r.center().y);
    let mut all = h.frame(vec![Event::PointerMoved(start)]);
    all.extend(h.frame(vec![Event::PointerButton { pos: start, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE }]));
    for i in 1..6 {
        all.extend(h.frame(vec![Event::PointerMoved(start + vec2(10.0 * i as f32, 0.0))]));
        // The plugin redraws (new revision) while the person drags.
        h.rev += 1;
    }
    assert!(all.is_empty(), "nothing is sent mid-drag: {all:?}");
    let end = start + vec2(50.0, 0.0);
    let done = h.frame(vec![Event::PointerButton { pos: end, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE }]);
    assert_eq!(done.len(), 1, "{done:?}");
    assert!(matches!(done[0].value, UiValue::Number(v) if v > 0.0));
    assert_eq!(done[0].rev, h.rev);
    assert!(h.frame(vec![]).is_empty());
}

#[test]
fn text_inputs_send_on_enter_and_number_fields_on_release() {
    let mut h = Harness::new(json!({"type":"column","children":[
        {"type":"text-input","id":"name","label":"Name","value":"","placeholder":"Name"},
        {"type":"number","id":"w","label":"Width","value":10,"min":0,"max":100,"unit":"px"}
    ]}));
    h.click("name");
    let typed = h.frame(vec![Event::Text("hello".into())]);
    assert!(typed.is_empty());
    let sent = h.frame(vec![Event::Key { key: Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE }]);
    assert_eq!(values(&sent), vec![("name".into(), UiValue::Text("hello".into()))]);
    let p = h.at("w");
    let mut e = h.frame(vec![Event::PointerMoved(p)]);
    e.extend(h.frame(vec![Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE }]));
    for i in 1..5 {
        e.extend(h.frame(vec![Event::PointerMoved(p + vec2(8.0 * i as f32, 0.0))]));
    }
    assert!(e.is_empty(), "{e:?}");
    let up =
        h.frame(vec![Event::PointerButton { pos: p + vec2(32.0, 0.0), button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE }]);
    assert!(matches!(values(&up).as_slice(), [(w, UiValue::Number(v))] if w == "w" && *v > 10.0), "{up:?}");
}

#[test]
fn canvas_sends_down_drag_up_with_button_mods_and_canvas_coordinates() {
    let mut h = Harness::new(json!({"type":"canvas","id":"pad","label":"Pad","width":200,"height":100,"cursor":"crosshair","ops":[
        {"op":"rect","min":[0,0],"max":[200,100],"radius":6,"fill":{"gradient":"linear","from":[0,0],"to":[200,0],"stops":[{"at":0,"rgba":[0,0,0,1]},{"at":1,"rgba":[1,1,1,1]}]}},
        {"op":"path","path":[{"move":[10,10]},{"line":[60,10]},{"line":[60,60]},{"line":[35,30]},{"line":[10,60]},"close"],"fill":[1,0,0,1],"stroke":[0,0,0,1],"stroke_width":2}
    ]}));
    let origin = h.host.rects["pad"].min;
    let shift = Modifiers { shift: true, ..Modifiers::NONE };
    let p = origin + vec2(20.0, 30.0);
    let mut e = h.frame(vec![Event::PointerMoved(p)]);
    assert!(e.is_empty(), "no move events without listen: {e:?}");
    e = h.frame_with(vec![Event::PointerButton { pos: p, button: PointerButton::Secondary, pressed: true, modifiers: shift }], shift);
    assert_eq!(e.len(), 1);
    let down = pointer(&e[0]);
    assert_eq!((down.kind, down.at, down.button), (PointerKind::Down, [20.0, 30.0], Some(PB::Secondary)));
    assert!(down.mods.shift);
    e = h.frame(vec![Event::PointerMoved(p + vec2(15.0, 5.0))]);
    let drag = pointer(&e[0]);
    assert_eq!((drag.kind, drag.at, drag.delta), (PointerKind::Drag, [35.0, 35.0], Some([15.0, 5.0])));
    assert!(h.frame(vec![]).is_empty(), "a still pointer sends nothing");
    e = h.frame(vec![Event::PointerButton {
        pos: p + vec2(15.0, 5.0),
        button: PointerButton::Secondary,
        pressed: false,
        modifiers: Modifiers::NONE,
    }]);
    assert_eq!(pointer(&e[0]).kind, PointerKind::Up);
    // A plain click is a down then an up (level 1.0 sent only the up).
    let kinds: Vec<_> = h.click_at(origin + vec2(5.0, 5.0), Modifiers::NONE).iter().map(|e| pointer(e).kind).collect();
    assert_eq!(kinds, vec![PointerKind::Down, PointerKind::Up]);
}

#[test]
fn canvas_opt_in_events_move_leave_wheel_double_click_and_keys() {
    let mut h = Harness::new(json!({"type":"column","children":[
        {"type":"canvas","id":"pad","label":"Pad","width":200,"height":100,"listen":["move","wheel","double-click","keys"],"ops":[]},
        {"type":"label","text":"below"}
    ]}));
    let origin = h.host.rects["pad"].min;
    let e = h.frame(vec![Event::PointerMoved(origin + vec2(10.0, 10.0))]);
    assert_eq!((pointer(&e[0]).kind, pointer(&e[0]).at), (PointerKind::Move, [10.0, 10.0]));
    let wheel = Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: vec2(0.0, -12.0),
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Wheel,
        modifiers: Modifiers::NONE,
    };
    let e = h.frame(vec![wheel]);
    let w = e.iter().map(pointer).find(|p| p.kind == PointerKind::Wheel).expect("wheel event");
    assert_eq!(w.delta, Some([0.0, -12.0]));
    let e = h.frame(vec![Event::PointerMoved(origin + vec2(10.0, 500.0))]);
    assert_eq!(pointer(&e[0]).kind, PointerKind::Leave);
    // Double click: two clicks in quick succession.
    let p = origin + vec2(50.0, 50.0);
    let all = h.double_click_at(p);
    assert!(all.iter().any(|e| pointer(e).kind == PointerKind::DoubleClick), "{all:?}");
    // The clicks gave the canvas focus: keys arrive; Escape releases focus and is not sent.
    let key = |key: Key, pressed: bool| Event::Key { key, physical_key: None, pressed, repeat: false, modifiers: Modifiers::NONE };
    let e = h.frame(vec![key(Key::ArrowLeft, true), key(Key::ArrowLeft, false)]);
    let ks: Vec<_> = e.iter().map(pointer).filter(|p| p.key.is_some()).map(|p| (p.kind, p.key.clone().unwrap_or_default())).collect();
    assert_eq!(ks, vec![(PointerKind::KeyDown, "ArrowLeft".to_string()), (PointerKind::KeyUp, "ArrowLeft".to_string())]);
    assert!(h.frame(vec![key(Key::Escape, true)]).iter().all(|e| pointer(e).key.is_none()));
    assert!(h.frame(vec![key(Key::A, true)]).iter().all(|e| pointer(e).key.is_none()), "no keys after Escape");
}

#[test]
fn tables_and_lists_select_rows_and_activate_on_double_click() {
    let mut h = Harness::new(json!({"type":"column","children":[
        {"type":"table","id":"t","label":"Layers","selection":"multi","columns":[{"label":"Name"},{"label":"Size","align":"right"}],
         "rows":[{"key":"a","cells":["Alpha","1"]},{"key":"b","cells":["Beta","2"]},{"key":"c","cells":["Gamma","3"]}]},
        {"type":"list","id":"l","label":"Presets","selection":"single","items":[{"key":"x","label":"Warm","icon":"sun"},{"key":"y","label":"Cool","detail":"blue"}]}
    ]}));
    let t = h.host.rects["t"];
    let row = |i: f32| pos2(t.left() + 30.0, t.top() + 6.0 + h.ctx.global_style().spacing.interact_size.y * (2.0 + i) + 4.0);
    let (r0, r2) = (row(0.0), row(2.0));
    assert_eq!(values(&h.click_at(r0, Modifiers::NONE)), vec![("t".into(), UiValue::Keys(vec!["a".into()]))]);
    let cmd = Modifiers { command: true, ctrl: true, ..Modifiers::NONE };
    assert_eq!(values(&h.click_at(r2, cmd)), vec![("t".into(), UiValue::Keys(vec!["a".into(), "c".into()]))]);
    let shift = Modifiers { shift: true, ..Modifiers::NONE };
    assert_eq!(values(&h.click_at(r0, shift)), vec![("t".into(), UiValue::Keys(vec!["a".into(), "b".into(), "c".into()]))]);
    let e = h.double_click_at(r2);
    assert!(values(&e).contains(&("t".into(), UiValue::Activate { activate: "c".into() })), "{e:?}");
    let l = h.host.rects["l"];
    let first = pos2(l.left() + 30.0, l.top() + h.ctx.global_style().spacing.interact_size.y * 1.5 + 4.0);
    assert_eq!(values(&h.click_at(first, Modifiers::NONE)), vec![("l".into(), UiValue::Keys(vec!["x".into()]))]);
}

#[test]
fn tabs_show_one_tab_and_collapsibles_report_toggles() {
    let mut h = Harness::new(json!({"type":"column","children":[
        {"type":"tabs","id":"tabs","tabs":[
            {"id":"a","label":"First","children":[{"type":"button","id":"in-a","label":"A"}]},
            {"id":"b","label":"Second","icon":"star","children":[{"type":"button","id":"in-b","label":"B"}]}
        ]},
        {"type":"collapsible","id":"more","label":"More","open":false,"children":[{"type":"button","id":"hidden","label":"H"}]}
    ]}));
    assert!(h.host.rects.contains_key("in-a") && !h.host.rects.contains_key("in-b"));
    assert_eq!(values(&h.click("tabs/b")), vec![("tabs".into(), UiValue::Text("b".into()))]);
    h.host.rects.clear();
    h.frame(vec![]);
    assert!(h.host.rects.contains_key("in-b") && !h.host.rects.contains_key("in-a"));
    assert!(!h.host.rects.contains_key("hidden"));
    assert_eq!(values(&h.click("more")), vec![("more".into(), UiValue::Bool(true))]);
    for _ in 0..10 {
        h.frame(vec![]);
    }
    assert!(h.host.rects.contains_key("hidden"));
}

fn form_tree(values: Value) -> Value {
    json!({"type":"form","hidden":["secret"],"values":values,"form":{"schema":1,"key":"f","title":"F","params":[
        {"key":"on","label":"On","kind":{"type":"bool","default":false}},
        {"key":"amount","label":"Amount","kind":{"type":"range","min":0,"max":10,"default":5},"unit":"percent",
         "visible_when":{"eq":{"key":"on","value":true}}},
        {"key":"mode","label":"Mode","kind":{"type":"radio","options":[{"id":"a","label":"A"},{"id":"b","label":"B"}],"default":"a"},
         "enabled_when":{"eq":{"key":"on","value":true}}},
        {"key":"secret","label":"Secret","kind":{"type":"text","default":"","multiline":false}},
        {"key":"tint","label":"Tint","kind":{"type":"color","default":[2.0,0.5,0.0,1.0],"alpha":false}},
        {"key":"at","label":"At","kind":{"type":"point","default":[1,2],"space":"image-percent"}}
    ]}})
}

#[test]
fn forms_follow_conditions_hidden_sets_and_send_typed_values_by_key() {
    let mut h = Harness::new(form_tree(json!({})));
    assert!(!h.host.rects.contains_key("amount"), "visible_when is false");
    assert!(!h.host.rects.contains_key("secret"), "hidden by the plugin");
    assert!(h.host.rects.contains_key("tint") && h.host.rects.contains_key("at"));
    // Disabled (enabled_when false): clicking the second radio sends nothing.
    let m = h.host.rects["mode"];
    assert!(h.click_at(pos2(m.right() - 8.0, m.center().y), Modifiers::NONE).is_empty());
    assert_eq!(values(&h.click("on")), vec![("on".into(), UiValue::Bool(true))]);
    // The plugin answers with a new tree: the condition now shows the slider.
    h.tree = form_tree(json!({"on": true}));
    h.rev += 1;
    h.frame(vec![]);
    h.frame(vec![]);
    assert!(h.host.rects.contains_key("amount"));
    let m = h.host.rects["mode"];
    let e = h.click_at(pos2(m.right() - 8.0, m.center().y), Modifiers::NONE);
    assert_eq!(values(&e), vec![("mode".into(), UiValue::Text("b".into()))], "choice ids, never labels");
}

#[test]
fn unknown_nodes_draw_a_notice_and_components_their_fallback() {
    let mut h = Harness::new(json!({"type":"column","children":[
        {"type":"hologram","depth":3},
        {"type":"chart","fallback":{"type":"button","id":"chart-fallback","label":"Open chart"}},
        {"type":"component","key":"h","name":"histogram","version":1,"props":{},"fallback":{"type":"button","id":"histo-fallback","label":"Histogram"}}
    ]}));
    assert!(h.host.rects.contains_key("chart-fallback") && h.host.rects.contains_key("histo-fallback"));
    assert_eq!(values(&h.click("histo-fallback")), vec![("histo-fallback".into(), UiValue::Click)]);
}

#[test]
fn every_level_1_1_node_and_draw_op_renders() {
    let img: Vec<u8> = [255u8, 0, 0, 255].repeat(4);
    let radial = json!({"gradient":"radial","center":[30,30],"radius":20,"stops":[{"at":0,"rgba":[1,1,1,1]},{"at":1,"rgba":[0,0,1,1]}]});
    let linear = json!({"gradient":"linear","from":[0,0],"to":[0,40],"stops":[{"at":0,"rgba":[1,0,0,1]},{"at":1,"rgba":[0,1,0,1]}]});
    let ops = json!([
        {"op":"clip","min":[0,0],"max":[60,80]},
        {"op":"circle","center":[30,30],"radius":20,"fill":radial,"stroke":[1,1,1,1],"stroke_width":2},
        {"op":"polygon","points":[[0,0],[40,0],[40,40],[20,10],[0,40]],"fill":linear},
        {"op":"reset-clip"},
        {"op":"path","path":[{"move":[70,10]},{"quad":[[90,0],[110,10]]},{"cubic":[[110,40],[70,40],[70,10]]}],"stroke":[1,1,0,1]},
        {"op":"image","image":"px","min":[90,50],"max":[110,70],"uv":[[0,0],[0.5,0.5]]},
        {"op":"text","at":[60,40],"text":"mid","size":11,"rgba":[1,1,1,1],"align":"center","valign":"center"},
        {"op":"line","from":[0,79],"to":[120,79],"width":1,"rgba":[1,1,1,1]}
    ]);
    let canvas = json!({"type":"canvas","id":"cv","label":"Canvas","width":120,"height":80,"ops":ops});
    let tree = json!({"type":"column","children":[
        {"type":"scroll","max_height":400,"children":[
            {"type":"grid","columns":2,"striped":true,"children":[{"type":"label","text":"A","style":"heading","icon":"info"},{"type":"spacer","size":4}]},
            {"type":"progress","value":0.25,"label":"Quarter"},{"type":"progress"},
            {"type":"radio","id":"r","label":"R","options":["x","y"],"selected":1,"horizontal":true},
            {"type":"select","id":"s","label":"S","options":["x","y"],"selected":0,"hint":"Pick"},
            {"type":"color","id":"c","label":"C","rgba":[1,0,0,1]},
            {"type":"text-area","id":"ta","label":"Notes","value":"a","rows":3,"mono":true},
            {"type":"icon","name":"not_in_this_build","label":"?"},
            {"type":"image","width":2,"height":2,"rgba":img,"name":"px","size":[8,8],"alt":"Red"},
            {"type":"image","width":2,"height":2,"rgba":img},
            {"type":"group","label":"G","children":[{"type":"separator"}]},
            canvas
        ]}
    ]});
    let mut h = Harness::new(tree);
    for id in ["r", "s", "c", "ta", "cv"] {
        assert!(h.host.rects.contains_key(id), "{id} not drawn");
    }
    assert!(h.frame(vec![]).is_empty());
}

#[test]
fn invalid_trees_draw_their_reason_and_parse_once_per_revision() {
    let mut h = Harness::new(json!({"type":"button","id":"","label":""}));
    assert!(h.host.rects.is_empty());
    assert!(h.frame(vec![]).is_empty());
    let cached = h.ui.trees.get("panel").map(|(rev, t)| (*rev, t.is_err()));
    assert_eq!(cached, Some((1, true)));
    h.tree = json!({"type":"button","id":"ok","label":"OK"});
    h.frame(vec![]);
    assert!(h.host.rects.is_empty(), "same revision: the cached tree is used");
    h.rev = 2;
    h.frame(vec![]);
    assert!(h.host.rects.contains_key("ok"));
    h.ui.forget("panel");
    assert!(h.ui.trees.is_empty());
}

#[test]
fn form_view_applies_patches_and_reports_live_edits() {
    let tree: craft_ext_core::ui::tree::Node = serde_json::from_value(form_tree(json!({}))).unwrap();
    let craft_ext_core::ui::tree::Node::Form(f) = tree else { panic!() };
    let mut view = FormView::new("dialog", f.form.clone(), &[("on".to_string(), craft_ext_core::form::Value::Bool(true))].into_iter().collect());
    assert_eq!(view.values().get("on"), Some(&craft_ext_core::form::Value::Bool(true)));
    view.apply_patch(&craft_ext_core::form::Patch::none(0).hide("tint").disable("mode")).unwrap();
    assert!(view.state().hidden.contains("tint") && view.state().disabled.contains("mode"));
    assert!(view.apply_patch(&craft_ext_core::form::Patch::none(0)).is_err(), "stale patch");
    let mut form = f.form.clone();
    form.params.retain(|p| p.key != "mode");
    view.replace_form(form);
    assert!(!view.state().disabled.contains("mode") && view.values().contains_key("on"));
    let ctx = egui::Context::default();
    let mut rec = Rec::default();
    for _ in 0..2 {
        ctx.run_ui(RawInput::default(), |root| {
            egui::CentralPanel::default().show(root, |ui| {
                view.show(ui, &mut rec);
            });
        })
        .drop_without_applying_deltas();
    }
    assert!(rec.rects.contains_key("amount") && !rec.rects.contains_key("tint"));
}

#[test]
fn overlays_paint_with_a_transform() {
    let ops: Vec<craft_ext_core::ui::tree::DrawOp> = serde_json::from_value(json!([
        {"op":"line","from":[-4000,0],"to":[4000,0],"width":1,"rgba":[1,0,0,1]},
        {"op":"circle","center":[0,0],"radius":3,"fill":[1,1,1,1]},
        {"op":"text","at":[10,10],"text":"10 × 20","size":12,"rgba":[1,1,1,1]}
    ]))
    .unwrap();
    craft_ext_core::ui::tree::validate_draw_list(&ops).unwrap();
    let ctx = egui::Context::default();
    ctx.run_ui(RawInput::default(), |root| {
        egui::CentralPanel::default().show(root, |ui| {
            paint_ops(ui.painter(), Xf::new(pos2(100.0, 100.0), 2.0), ui.max_rect(), &ops, &|_| None);
        });
    })
    .drop_without_applying_deltas();
}
