use super::*;
use serde_json::json;

/// Level-1.0 trees as plugins send them today (the SDK's builders, the examples, the old tests).
fn trees_1_0() -> Vec<Value> {
    let form = json!({"schema":1,"key":"shapes","title":"Shapes","params":[
        {"key":"count","label":"Count","kind":{"type":"int","min":1,"max":20,"default":3}},
        {"key":"kind","label":"Kind","kind":{"type":"choice","options":[{"id":"rect","label":"Rectangle"},{"id":"oval","label":"Oval"}],"default":"rect"}}
    ]});
    vec![
        json!({"type":"column","children":[{"type":"label","text":"Hi"},{"type":"button","id":"go","label":"Go"},
            {"type":"slider","id":"s","label":"Size","value":2,"min":0,"max":10}]}),
        json!({"type":"column","children":[{"type":"label","text":"Selected layers"},{"type":"label","text":"• Layer 1"},
            {"type":"separator"},{"type":"button","id":"add-marker","label":"Add note marker"}]}),
        json!({"type":"column","children":[{"type":"form","form":form,"values":{"count":4}},{"type":"button","id":"create","label":"Create shapes"}]}),
        json!({"type":"row","children":[
            {"type":"checkbox","id":"c","label":"On","value":true},
            {"type":"text-input","id":"t","label":"Name","value":"x"},
            {"type":"select","id":"m","label":"Mode","options":["A","B"],"selected":1},
            {"type":"color","id":"col","label":"Tint","rgba":[1.0,0.5,0.0,1.0]},
            {"type":"image","width":1,"height":1,"rgba":[1,2,3,255]},
            {"type":"group","label":"G","children":[]}
        ]}),
        json!({"type":"canvas","id":"plot","label":"Plot","width":100.0,"height":50.0,"ops":[
            {"op":"line","from":[0,0],"to":[10,10],"width":1.0,"rgba":[1,1,1,1]},
            {"op":"rect","min":[0,0],"max":[5,5],"fill":[0,0,0,0.5],"stroke":[1,0,0,1]},
            {"op":"rect","min":[-4000,0],"max":[4000,5],"fill":[0,0,0,0]},
            {"op":"circle","center":[5,5],"radius":2,"fill":[1,0,0,1]},
            {"op":"text","at":[1,1],"text":"10 × 20","size":12,"rgba":[1,1,1,1]}
        ]}),
        json!({"type":"component","key":"h","name":"histogram","version":1,"props":{},"fallback":{"type":"label","text":"No histogram"}}),
    ]
}

/// A tree using every level-1.1 node and draw op.
fn rich() -> Value {
    let table = json!({"type":"table","id":"t","label":"Layers","columns":[{"label":"Name"},{"label":"Size","width":60,"align":"right"}],
        "rows":[{"key":"r1","cells":["One","10"]},{"key":"r2","cells":["Two"]}],"selection":"multi","selected":["r2"],"max_height":200});
    let list =
        json!({"type":"list","id":"l","label":"Presets","items":[{"key":"p1","label":"Warm","icon":"sun","detail":"Warmer"}],"selection":"single"});
    let gradient = json!({"gradient":"linear","from":[0,0],"to":[50,0],"stops":[{"at":0,"rgba":[1,0,0,1]},{"at":1,"rgba":[0,0,1,1]}]});
    let radial = json!({"gradient":"radial","center":[20,20],"radius":5,"stops":[{"at":0,"rgba":[1,1,1,1]},{"at":1,"rgba":[1,1,1,0]}]});
    let path = json!([{"move":[0,0]},{"line":[10,0]},{"quad":[[15,5],[10,10]]},{"cubic":[[5,15],[0,15],[0,10]]},"close"]);
    let ops = json!([
        {"op":"clip","min":[0,0],"max":[100,100]},
        {"op":"rect","min":[0,0],"max":[50,50],"radius":4,"stroke":[1,1,1,1],"stroke_width":2,"fill":gradient},
        {"op":"circle","center":[20,20],"radius":5,"fill":radial,"stroke":[0,0,0,1]},
        {"op":"path","path":path,"fill":[0,1,0,1],"stroke":[0,0,0,1],"stroke_width":1.5},
        {"op":"polygon","points":[[0,0],[10,0],[5,8]],"fill":[1,1,0,1]},
        {"op":"image","image":"swatch","min":[60,0],"max":[80,20],"tint":[1,1,1,0.5]},
        {"op":"reset-clip"},
        {"op":"text","at":[100,50],"text":"centre","size":12,"rgba":[1,1,1,1],"align":"center","valign":"center","mono":true}
    ]);
    let canvas = json!({"type":"canvas","id":"c","label":"Pad","width":200,"height":100,"listen":["move","wheel","double-click","keys"],
        "cursor":"crosshair","hint":"Draw here","ops":ops});
    let image = json!({"type":"image","width":1,"height":1,"rgba":[255,0,0,255],"name":"swatch","hidden":true});
    let tabs = json!({"type":"tabs","id":"tabs","selected":"b","tabs":[
        {"id":"a","label":"Layers","icon":"layers","children":[table, list]},
        {"id":"b","label":"Draw","children":[canvas, image]}
    ]});
    let more = json!({"type":"collapsible","id":"more","label":"More","open":true,"children":[
        {"type":"scroll","max_height":120,"children":[{"type":"label","text":"Long","style":"heading","icon":"info"}]},
        {"type":"grid","columns":2,"striped":true,"children":[{"type":"label","text":"A"},{"type":"spacer","size":8}]}
    ]});
    let controls = vec![
        json!({"type":"number","id":"n","label":"Width","value":10,"min":0,"max":100,"step":0.5,"decimals":1,"unit":"px","hint":"Width in pixels"}),
        json!({"type":"text-area","id":"notes","label":"Notes","value":"a\nb","rows":4,"placeholder":"Type…","mono":false}),
        json!({"type":"progress","value":0.5,"label":"Half"}),
        json!({"type":"progress"}),
        json!({"type":"toggle","id":"live","label":"Live","value":true,"disabled":true}),
        json!({"type":"radio","id":"r","label":"Mode","options":["A","B"],"selected":0,"horizontal":true}),
        json!({"type":"icon","name":"star","size":16,"label":"Favourite"}),
        json!({"type":"button","id":"del","label":"Delete","icon":"trash","icon_only":true,"hint":"Delete the selection"}),
    ];
    let mut children = vec![tabs, more];
    children.extend(controls);
    json!({"type":"column","children":children})
}

#[test]
fn every_1_0_tree_parses_validates_round_trips_and_is_level_1_0() {
    for json in trees_1_0() {
        let old: super::super::Node = serde_json::from_value(json.clone()).unwrap();
        old.validate().unwrap();
        let n: Node = serde_json::from_value(json.clone()).unwrap();
        n.validate().unwrap();
        assert_eq!(n.level(), UI_LEVEL_1_0, "{json}");
        // Serialising the new type gives JSON the 1.0 parser reads back to the same tree.
        let again: super::super::Node = serde_json::from_value(serde_json::to_value(&n).unwrap()).unwrap();
        assert_eq!(again, old);
        assert_eq!(Node::from(old), n);
        n.validate_at(UI_LEVEL_1_0).unwrap();
    }
}

#[test]
fn rich_tree_round_trips_and_needs_1_1() {
    let n = Node::parse(&rich()).unwrap();
    let back: Node = serde_json::from_value(serde_json::to_value(&n).unwrap()).unwrap();
    assert_eq!(back, n);
    assert_eq!(n.level(), UI_LEVEL);
    assert_eq!(n.first_1_1_feature(), Some("tabs"));
    let err = n.validate_at(UI_LEVEL_1_0).unwrap_err();
    assert!(err.0.contains("needs ui 1.1"), "{err}");
    n.validate_at(UI_LEVEL).unwrap();
    // A new field on an old node is level 1.1 too.
    let hinted = Node::parse(&json!({"type":"button","id":"b","label":"B","hint":"Tip"})).unwrap();
    assert_eq!(hinted.level(), UI_LEVEL);
    assert_eq!(hinted.first_1_1_feature(), None);
    assert!(hinted.validate_at(UI_LEVEL_1_0).unwrap_err().0.contains("level 1.1 fields"));
}

#[test]
fn session_contract_minor_carries_the_ui_level() {
    assert_eq!(ui_level_for_session(Level::new(1, 0)), UI_LEVEL_1_0);
    assert_eq!(ui_level_for_session(Level::new(1, 1)), UI_LEVEL);
    assert_eq!(ui_level_for_session(Level::new(1, 4)), UI_LEVEL);
}

#[test]
fn unknown_nodes_degrade_and_broken_known_nodes_explain_themselves() {
    let json = json!({"type":"column","children":[
        {"type":"webview","url":"https://example.com"},
        {"type":"chart","series":[1,2],"fallback":{"type":"label","text":"No chart"}}
    ]});
    let n = Node::parse(&json).unwrap();
    let Node::Column(s) = &n else { panic!() };
    assert!(matches!(&s.children[0], Node::Unknown(u) if u.kind == "webview" && u.fallback.is_none()));
    assert!(
        matches!(&s.children[1], Node::Unknown(u) if u.fallback.as_deref() == Some(&Node::Label(Label { text: "No chart".into(), style: TextStyle::Body, icon: None })))
    );
    // The unknown node's own fields are not echoed back to the frontend.
    assert_eq!(serde_json::to_value(&s.children[0]).unwrap(), json!({"type":"webview"}));
    assert_eq!(n.level(), UI_LEVEL_1_0);
    // An unknown node's fallback is validated like any other node.
    assert!(Node::parse(&json!({"type":"chart","fallback":{"type":"button","id":"","label":""}})).is_err());
    // A known type with bad fields is refused with serde's reason, not drawn as "unknown".
    let err = Node::parse(&json!({"type":"label","text":"x","colour":"red"})).unwrap_err();
    assert!(err.0.contains("unknown field `colour`"), "{err}");
    let err = Node::parse(&json!({"type":"slider","id":"s","label":"S","value":1})).unwrap_err();
    assert!(err.0.contains("missing field `min`"), "{err}");
    assert!(serde_json::from_value::<Node>(json!({"text":"no type"})).is_err());
}

fn refused(json: Value) -> String {
    match Node::parse(&json) {
        Ok(_) => panic!("accepted {json}"),
        Err(e) => e.0,
    }
}

#[test]
fn limits_are_enforced() {
    // Controls need an id and a visible label; hints are checked text.
    refused(json!({"type":"toggle","id":"t","label":" ","value":true}));
    refused(json!({"type":"number","id":"","label":"N","value":1}));
    refused(json!({"type":"button","id":"b","label":"B","hint":"\u{202e}evil"}));
    refused(json!({"type":"button","id":"b","label":"B","icon_only":true}));
    refused(json!({"type":"icon","name":"Not-An-Icon"}));
    // Numbers.
    refused(json!({"type":"number","id":"n","label":"N","value":1,"min":5,"max":1}));
    refused(json!({"type":"number","id":"n","label":"N","value":1,"decimals":11}));
    refused(json!({"type":"slider","id":"s","label":"S","value":1,"min":0,"max":2,"step":0}));
    refused(json!({"type":"progress","value":1.5}));
    refused(json!({"type":"spacer","size":-1}));
    refused(json!({"type":"text-area","id":"a","label":"A","value":"","rows":0}));
    refused(json!({"type":"text-area","id":"a","label":"A","value":"x".repeat(MAX_TEXT_AREA + 1)}));
    // Layout.
    refused(json!({"type":"grid","columns":0,"children":[]}));
    refused(json!({"type":"grid","columns":17,"children":[]}));
    refused(json!({"type":"tabs","id":"t","tabs":[]}));
    refused(json!({"type":"tabs","id":"t","tabs":[{"id":"a","label":"A","children":[]},{"id":"a","label":"B","children":[]}]}));
    refused(json!({"type":"tabs","id":"t","selected":"z","tabs":[{"id":"a","label":"A","children":[]}]}));
    refused(json!({"type":"scroll","max_height":0,"children":[]}));
    // Tables and lists.
    let cols = json!([{"label":"A"}]);
    refused(json!({"type":"table","id":"t","label":"T","columns":[],"rows":[]}));
    refused(json!({"type":"table","id":"t","label":"T","columns":cols,"rows":[{"key":"a","cells":["1","2"]}]}));
    refused(json!({"type":"table","id":"t","label":"T","columns":cols,"rows":[{"key":"a","cells":[]},{"key":"a","cells":[]}]}));
    refused(json!({"type":"table","id":"t","label":"T","columns":cols,"rows":[{"key":"a","cells":[]}],"selected":["a"]}));
    refused(json!({"type":"list","id":"l","label":"L","items":[{"key":"a","label":"A"}],"selection":"single","selected":["b"]}));
    let rows: Vec<Value> = (0..MAX_ROWS + 1).map(|i| json!({"key": format!("r{i}"), "cells": []})).collect();
    refused(json!({"type":"table","id":"t","label":"T","columns":cols,"rows":rows}));
    // Twenty tables of 10 000 rows × 1 column pass one by one but not together.
    let rows: Vec<Value> = (0..MAX_ROWS).map(|i| json!({"key": format!("r{i}"), "cells": []})).collect();
    let tables: Vec<Value> = (0..11).map(|i| json!({"type":"table","id":format!("t{i}"),"label":"T","columns":cols,"rows":rows})).collect();
    assert!(refused(json!({"type":"column","children":tables})).contains("too large"));
}

fn canvas(ops: Value) -> Value {
    json!({"type":"canvas","id":"c","label":"C","width":10,"height":10,"ops":ops})
}

#[test]
fn draw_limits_are_enforced() {
    let stop = |at: f32| json!({"at":at,"rgba":[1,1,1,1]});
    refused(canvas(json!([{"op":"rect","min":[0,0],"max":[1,1],"fill":{"gradient":"linear","from":[0,0],"to":[1,0],"stops":[stop(0.0)]}}])));
    refused(canvas(
        json!([{"op":"rect","min":[0,0],"max":[1,1],"fill":{"gradient":"linear","from":[0,0],"to":[1,0],"stops":[stop(0.8), stop(0.2)]}}]),
    ));
    let many: Vec<Value> = (0..17).map(|i| stop(i as f32 / 17.0)).collect();
    refused(canvas(json!([{"op":"rect","min":[0,0],"max":[1,1],"fill":{"gradient":"radial","center":[0,0],"radius":1,"stops":many}}])));
    refused(canvas(json!([{"op":"line","from":[0,0],"to":[1,1],"width":1000,"rgba":[1,1,1,1]}])));
    refused(canvas(json!([{"op":"rect","min":[0,0],"max":[1,1],"fill":[1,1,1,1],"radius":-1}])));
    refused(canvas(json!([{"op":"polygon","points":[[0,0],[1,1]]}])));
    refused(canvas(json!([{"op":"text","at":[0,0],"text":"x","size":0,"rgba":[1,1,1,1]}])));
    refused(canvas(json!([{"op":"image","image":"missing","min":[0,0],"max":[1,1]}])));
    refused(canvas(json!([{"op":"image","image":"x","min":[0,0],"max":[1,1],"uv":[[0,0],[2,1]]}])));
    let long: Vec<Value> = (0..=MAX_PATH_POINTS).map(|_| json!({"line":[0,0]})).collect();
    assert!(refused(canvas(json!([{"op":"path","path":long}]))).contains("paths are too large"));
    // Twenty-five 4 096-point paths pass one by one but not together.
    let path: Vec<Value> = (0..MAX_PATH_POINTS).map(|_| json!({"line":[0,0]})).collect();
    let ops: Vec<Value> = (0..25).map(|_| json!({"op":"path","path":path})).collect();
    refused(canvas(Value::Array(ops)));
    // Images: names are unique and resolved, total bytes are capped.
    let img = |name: &str| json!({"type":"image","width":1,"height":1,"rgba":[0,0,0,0],"name":name,"hidden":true});
    refused(json!({"type":"column","children":[img("a"), img("a")]}));
    Node::parse(&json!({"type":"column","children":[img("a"), canvas(json!([{"op":"image","image":"a","min":[0,0],"max":[1,1]}]))]})).unwrap();
    // Overlays use the same op checks and cannot draw images.
    let ops: Vec<DrawOp> = serde_json::from_value(json!([{"op":"circle","center":[0,0],"radius":1,"fill":[1,1,1,1]}])).unwrap();
    validate_draw_list(&ops).unwrap();
    let ops: Vec<DrawOp> = serde_json::from_value(json!([{"op":"image","image":"a","min":[0,0],"max":[1,1]}])).unwrap();
    assert!(validate_draw_list(&ops).is_err());
}

#[test]
fn gradients_sample_between_stops() {
    let g = Gradient::Linear {
        from: [0.0, 0.0],
        to: [10.0, 0.0],
        stops: vec![Stop { at: 0.0, rgba: [0.0, 0.0, 0.0, 1.0] }, Stop { at: 1.0, rgba: [1.0, 1.0, 1.0, 1.0] }],
    };
    assert_eq!(g.sample([5.0, 3.0]), [0.5, 0.5, 0.5, 1.0]);
    assert_eq!(g.sample([-5.0, 0.0]), [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(g.sample([50.0, 0.0]), [1.0, 1.0, 1.0, 1.0]);
    let r = Gradient::Radial { center: [0.0, 0.0], radius: 2.0, stops: g.stops().to_vec() };
    assert_eq!(r.sample([1.0, 0.0]), [0.5, 0.5, 0.5, 1.0]);
}

#[test]
fn event_values_have_stable_json() {
    let ev = |value: UiValue| serde_json::to_value(UiEvent { surface: "s".into(), rev: 3, widget: "w".into(), value }).unwrap()["value"].clone();
    assert_eq!(ev(UiValue::Click), Value::Null);
    assert_eq!(ev(UiValue::Keys(vec!["a".into()])), json!(["a"]));
    assert_eq!(ev(UiValue::Activate { activate: "r1".into() }), json!({"activate":"r1"}));
    // A plain down event is exactly the level-1.0 shape.
    let down = PointerEvent { kind: PointerKind::Down, at: [1.0, 2.0], button: None, mods: Mods::default(), delta: None, key: None };
    assert_eq!(ev(UiValue::Pointer(down.clone())), json!({"kind":"down","at":[1.0,2.0]}));
    let wheel = PointerEvent { kind: PointerKind::Wheel, delta: Some([0.0, -3.0]), mods: Mods { shift: true, ..Mods::default() }, ..down.clone() };
    assert_eq!(ev(UiValue::Pointer(wheel)), json!({"kind":"wheel","at":[1.0,2.0],"mods":{"shift":true},"delta":[0.0,-3.0]}));
    let key = PointerEvent { kind: PointerKind::KeyDown, key: Some("ArrowLeft".into()), ..down };
    assert_eq!(ev(UiValue::Pointer(key)), json!({"kind":"key-down","at":[1.0,2.0],"key":"ArrowLeft"}));
    // Parsing keeps the shapes apart.
    let parsed: UiValue = serde_json::from_value(json!({"kind":"double-click","at":[0,0],"button":"primary"})).unwrap();
    assert!(matches!(parsed, UiValue::Pointer(PointerEvent { kind: PointerKind::DoubleClick, button: Some(PointerButton::Primary), .. })));
    assert_eq!(serde_json::from_value::<UiValue>(json!({"activate":"x"})).unwrap(), UiValue::Activate { activate: "x".into() });
    assert_eq!(serde_json::from_value::<UiValue>(json!(["x"])).unwrap(), UiValue::Keys(vec!["x".into()]));
    let e = UiEvent { surface: "s".into(), rev: 3, widget: "w".into(), value: UiValue::Click };
    assert!(e.is_stale(4) && !e.is_stale(3));
}

#[test]
fn icon_list_is_sorted_and_valid() {
    assert!(ICONS.windows(2).all(|w| w[0] < w[1]));
    for name in ICONS {
        icon_name(Some(name)).unwrap();
    }
}
