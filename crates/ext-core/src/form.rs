//! Versioned, renderer-independent forms. Colors are straight RGBA in the host working space
//! (including HDR RGB); alpha is in 0..=1. Angles are degrees. Invalid values are refused, never clamped.
use crate::text::{Lines, validate_text};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const FORM_SCHEMA: u32 = 1;
pub const MAX_PARAMS: usize = 128;
pub const MAX_COND_DEPTH: usize = 16;
pub const MAX_COND_NODES: usize = 256;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "WordWire")]
pub struct Word {
    pub id: Option<String>,
    pub english: String,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum WordWire {
    Plain(String),
    Full(WordObject),
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WordObject {
    id: Option<String>,
    english: String,
}
impl TryFrom<WordWire> for Word {
    type Error = String;
    fn try_from(w: WordWire) -> Result<Self, String> {
        Ok(match w {
            WordWire::Plain(english) => Self { id: None, english },
            WordWire::Full(WordObject { id, english }) => Self { id, english },
        })
    }
}
impl From<&str> for Word {
    fn from(s: &str) -> Self {
        Self { id: None, english: s.into() }
    }
}
impl Word {
    fn validate(&self) -> Result<(), String> {
        validate_text(&self.english, 4000, Lines::One).map_err(|e| e.to_string())?;
        if let Some(id) = &self.id {
            validate_text(id, 256, Lines::One).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Form {
    pub schema: u32,
    pub key: String,
    pub title: Word,
    pub params: Vec<Param>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<Preview>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preview {
    pub enabled: bool,
    pub compare: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Param {
    pub key: String,
    pub label: Word,
    pub kind: ParamKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<Unit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible_when: Option<Cond>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled_when: Option<Cond>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help: Option<Word>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ParamKind {
    Section,
    Range {
        min: f64,
        max: f64,
        default: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        step: Option<f64>,
    },
    Int {
        min: i64,
        max: i64,
        default: i64,
    },
    Number {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
        default: f64,
    },
    Bool {
        default: bool,
    },
    Choice {
        options: Vec<Choice>,
        default: String,
    },
    Radio {
        options: Vec<Choice>,
        default: String,
    },
    Angle {
        default: f64,
    },
    Color {
        default: [f32; 4],
        alpha: bool,
    },
    Point {
        default: [f64; 2],
        space: PointSpace,
    },
    Text {
        default: String,
        multiline: bool,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub id: String,
    pub label: Word,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PointSpace {
    LayerPixels,
    ImagePercent,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Unit {
    Pixels,
    Percent,
    Degrees,
    Seconds,
    Frames,
    Custom(String),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Number(f64),
    Bool(bool),
    String(String),
    Color([f32; 4]),
    Point([f64; 2]),
}
pub type Values = BTreeMap<String, Value>;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum Cond {
    Eq { key: String, value: Value },
    All(Vec<Cond>),
    Any(Vec<Cond>),
    Not(Box<Cond>),
}
impl Cond {
    fn bounded(&self, depth: usize, n: &mut usize) -> bool {
        *n += 1;
        if depth > MAX_COND_DEPTH || *n > MAX_COND_NODES {
            return false;
        }
        match self {
            Self::Eq { .. } => true,
            Self::All(cs) | Self::Any(cs) => cs.iter().all(|c| c.bounded(depth + 1, n)),
            Self::Not(c) => c.bounded(depth + 1, n),
        }
    }
    pub fn eval(&self, values: &Values) -> bool {
        if !self.bounded(0, &mut 0) {
            return false;
        }
        self.eval_inner(values)
    }
    fn eval_inner(&self, v: &Values) -> bool {
        match self {
            Self::Eq { key, value } => v.get(key) == Some(value),
            Self::All(cs) => cs.iter().all(|c| c.eval_inner(v)),
            Self::Any(cs) => cs.iter().any(|c| c.eval_inner(v)),
            Self::Not(c) => !c.eval_inner(v),
        }
    }
    fn validate(&self, f: &Form) -> Result<(), String> {
        if !self.bounded(0, &mut 0) {
            return Err("condition limit exceeded".into());
        }
        match self {
            Self::Eq { key, value } => f.param(key)?.accept(value),
            Self::All(cs) | Self::Any(cs) => {
                for c in cs {
                    c.validate(f)?
                }
                Ok(())
            }
            Self::Not(c) => c.validate(f),
        }
    }
}
fn key(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64 && s.bytes().all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
}
impl Param {
    fn default_value(&self) -> Option<Value> {
        Some(match &self.kind {
            ParamKind::Section => return None,
            ParamKind::Range { default, .. } | ParamKind::Number { default, .. } | ParamKind::Angle { default } => Value::Number(*default),
            ParamKind::Int { default, .. } => Value::Number(*default as f64),
            ParamKind::Bool { default } => Value::Bool(*default),
            ParamKind::Choice { default, .. } | ParamKind::Radio { default, .. } | ParamKind::Text { default, .. } => Value::String(default.clone()),
            ParamKind::Color { default, .. } => Value::Color(*default),
            ParamKind::Point { default, .. } => Value::Point(*default),
        })
    }
    fn accept(&self, v: &Value) -> Result<(), String> {
        let ok = match (&self.kind, v) {
            (ParamKind::Range { min, max, .. }, Value::Number(n)) => n.is_finite() && n >= min && n <= max,
            (ParamKind::Int { min, max, .. }, Value::Number(n)) => n.is_finite() && n.fract() == 0.0 && *n >= *min as f64 && *n <= *max as f64,
            (ParamKind::Number { min, max, .. }, Value::Number(n)) => n.is_finite() && min.is_none_or(|m| *n >= m) && max.is_none_or(|m| *n <= m),
            (ParamKind::Angle { .. }, Value::Number(n)) => n.is_finite(),
            (ParamKind::Bool { .. }, Value::Bool(_)) => true,
            (ParamKind::Choice { options, .. } | ParamKind::Radio { options, .. }, Value::String(s)) => options.iter().any(|o| o.id == *s),
            (ParamKind::Color { alpha, .. }, Value::Color(c)) => {
                c.iter().all(|n| n.is_finite()) && (0.0..=1.0).contains(&c[3]) && (*alpha || c[3] == 1.0)
            }
            (ParamKind::Point { .. }, Value::Point(p)) => p.iter().all(|n| n.is_finite()),
            (ParamKind::Text { multiline, .. }, Value::String(s)) => {
                s.is_empty() || validate_text(s, 16384, if *multiline { Lines::Many } else { Lines::One }).is_ok()
            }
            _ => false,
        };
        if ok { Ok(()) } else { Err(format!("invalid value for {}", self.key)) }
    }
}
impl Form {
    fn param(&self, k: &str) -> Result<&Param, String> {
        self.params.iter().find(|p| p.key == k).ok_or_else(|| format!("unknown parameter {k}"))
    }
    pub fn defaults(&self) -> Values {
        self.params.iter().filter_map(|p| p.default_value().map(|v| (p.key.clone(), v))).collect()
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != FORM_SCHEMA || !key(&self.key) || self.params.len() > MAX_PARAMS {
            return Err("invalid form schema, key or size".into());
        }
        self.title.validate()?;
        let mut keys = BTreeSet::new();
        for p in &self.params {
            if !key(&p.key) || !keys.insert(&p.key) {
                return Err("invalid or duplicate parameter key".into());
            }
            p.label.validate()?;
            if let Some(h) = &p.help {
                h.validate()?
            }
            if let Some(Unit::Custom(s)) = &p.unit {
                validate_text(s, 64, Lines::One).map_err(|e| e.to_string())?
            }
            match &p.kind {
                ParamKind::Range { min, max, step, .. } => {
                    if !min.is_finite() || !max.is_finite() || min >= max || step.is_some_and(|s| !s.is_finite() || s <= 0.0) {
                        return Err("invalid range".into());
                    }
                }
                ParamKind::Int { min, max, .. } => {
                    if min > max || *min < -9007199254740991 || *max > 9007199254740991 {
                        return Err("invalid integer range".into());
                    }
                }
                ParamKind::Number { min, max, .. } => {
                    if min.is_some_and(|n| !n.is_finite()) || max.is_some_and(|n| !n.is_finite()) || min.zip(*max).is_some_and(|(a, b)| a > b) {
                        return Err("invalid number range".into());
                    }
                }
                ParamKind::Choice { options, .. } | ParamKind::Radio { options, .. } => {
                    let mut ids = BTreeSet::new();
                    if options.is_empty() || options.len() > 128 {
                        return Err("invalid choices".into());
                    }
                    for o in options {
                        if !key(&o.id) || !ids.insert(&o.id) {
                            return Err("invalid choice id".into());
                        }
                        o.label.validate()?
                    }
                }
                _ => (),
            }
            if let Some(v) = p.default_value() {
                p.accept(&v)?
            }
            for c in p.visible_when.iter().chain(p.enabled_when.iter()) {
                c.validate(self)?
            }
        }
        Ok(())
    }
    /// Validates a partial map; unknown keys and out-of-range values are refused.
    pub fn validate_values(&self, v: &Values) -> Result<(), String> {
        self.validate()?;
        for (k, v) in v {
            self.param(k)?.accept(v)?
        }
        Ok(())
    }
    pub fn apply_patch(&self, state: &mut FormState, patch: &Patch) -> Result<(), String> {
        self.validate_values(&state.values)?;
        if patch.schema != FORM_SCHEMA || patch.base_rev != state.rev || patch.rev <= patch.base_rev || patch.ops.len() > 512 {
            return Err("invalid or stale patch".into());
        }
        let mut next = state.clone();
        for op in &patch.ops {
            let k = op.key();
            let p = self.param(k)?;
            if matches!(p.kind, ParamKind::Section) && matches!(op, PatchOp::Set { .. } | PatchOp::Reset { .. }) {
                return Err("section has no value".into());
            }
            match op {
                PatchOp::Set { key, value } => {
                    p.accept(value)?;
                    next.values.insert(key.clone(), value.clone());
                }
                PatchOp::Reset { key } => {
                    if let Some(v) = p.default_value() {
                        next.values.insert(key.clone(), v);
                    }
                }
                PatchOp::Disable { key } => {
                    next.disabled.insert(key.clone());
                }
                PatchOp::Enable { key } => {
                    next.disabled.remove(key);
                }
                PatchOp::Hide { key } => {
                    next.hidden.insert(key.clone());
                }
                PatchOp::Show { key } => {
                    next.hidden.remove(key);
                }
            }
        }
        next.rev = patch.rev;
        *state = next;
        Ok(())
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FormState {
    pub rev: u64,
    pub values: Values,
    pub disabled: BTreeSet<String>,
    pub hidden: BTreeSet<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Patch {
    pub schema: u32,
    pub base_rev: u64,
    pub rev: u64,
    pub ops: Vec<PatchOp>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case", deny_unknown_fields)]
pub enum PatchOp {
    Set { key: String, value: Value },
    Reset { key: String },
    Disable { key: String },
    Enable { key: String },
    Hide { key: String },
    Show { key: String },
}
impl PatchOp {
    fn key(&self) -> &str {
        match self {
            Self::Set { key, .. } | Self::Reset { key } | Self::Disable { key } | Self::Enable { key } | Self::Hide { key } | Self::Show { key } => {
                key
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn vignette() -> Form {
        serde_json::from_str(r#"{"schema":1,"key":"vignette","title":"Vignette","params":[{"key":"amount","label":"Amount","kind":{"type":"range","min":-100,"max":100,"default":0},"unit":"percent"},{"key":"shape","label":"Shape","kind":{"type":"choice","options":[{"id":"round","label":"Round"},{"id":"oval","label":"Oval"}],"default":"round"}}]}"#).unwrap()
    }
    #[test]
    fn roundtrip() {
        let f = vignette();
        f.validate().unwrap();
        assert_eq!(f, serde_json::from_slice::<Form>(&serde_json::to_vec(&f).unwrap()).unwrap());
        assert_eq!(f.defaults()["shape"], Value::String("round".into()));
    }
    #[test]
    fn atomic_and_stale() {
        let f = vignette();
        let mut s = FormState { values: f.defaults(), ..Default::default() };
        let before = s.clone();
        let mut p = Patch {
            schema: 1,
            base_rev: 0,
            rev: 1,
            ops: vec![PatchOp::Hide { key: "amount".into() }, PatchOp::Set { key: "amount".into(), value: Value::Number(101.0) }],
        };
        assert!(f.apply_patch(&mut s, &p).is_err());
        assert_eq!(s, before);
        p.ops.pop();
        f.apply_patch(&mut s, &p).unwrap();
        assert!(f.apply_patch(&mut s, &p).is_err());
    }
    #[test]
    fn rejects_bad_schema_keys_text_conditions() {
        let mut f = vignette();
        f.params.push(f.params[0].clone());
        assert!(f.validate().is_err());
        f.params.pop();
        f.params[0].label = "\u{202e}bad".into();
        assert!(f.validate().is_err());
        let mut c = Cond::All(vec![]);
        for _ in 0..20 {
            c = Cond::Not(Box::new(c));
        }
        assert!(!c.eval(&Values::new()));
    }
}
impl Patch {
    /// Empty update at the caller's revision. All builders advance the revision once.
    pub fn none(base_rev: u64) -> Self {
        Self { schema: FORM_SCHEMA, base_rev, rev: base_rev.saturating_add(1), ops: vec![] }
    }
    pub fn set(mut self, key: impl Into<String>, value: Value) -> Self {
        self.ops.push(PatchOp::Set { key: key.into(), value });
        self
    }
    pub fn disable(mut self, key: impl Into<String>) -> Self {
        self.ops.push(PatchOp::Disable { key: key.into() });
        self
    }
    pub fn enable(mut self, key: impl Into<String>) -> Self {
        self.ops.push(PatchOp::Enable { key: key.into() });
        self
    }
    pub fn hide(mut self, key: impl Into<String>) -> Self {
        self.ops.push(PatchOp::Hide { key: key.into() });
        self
    }
    pub fn show(mut self, key: impl Into<String>) -> Self {
        self.ops.push(PatchOp::Show { key: key.into() });
        self
    }
}
#[cfg(test)]
mod boundary_tests {
    use super::*;
    fn form(kind: ParamKind) -> Form {
        Form {
            schema: 1,
            key: "form".into(),
            title: "Form".into(),
            preview: None,
            params: vec![Param { key: "value".into(), label: "Value".into(), kind, unit: None, help: None, visible_when: None, enabled_when: None }],
        }
    }
    #[test]
    fn every_kind_roundtrips_and_validates_defaults() {
        for kind in [
            ParamKind::Section,
            ParamKind::Range { min: 0.0, max: 1.0, default: 0.5, step: Some(0.1) },
            ParamKind::Int { min: 0, max: 10, default: 1 },
            ParamKind::Number { min: None, max: Some(10.0), default: 2.0 },
            ParamKind::Bool { default: true },
            ParamKind::Radio { options: vec![Choice { id: "a".into(), label: "A".into() }], default: "a".into() },
            ParamKind::Angle { default: 45.0 },
            ParamKind::Color { default: [2.0, 0.0, 0.0, 1.0], alpha: false },
            ParamKind::Point { default: [1.0, 2.0], space: PointSpace::LayerPixels },
            ParamKind::Text { default: "hello\nworld".into(), multiline: true },
        ] {
            let f = form(kind);
            f.validate_values(&f.defaults()).unwrap();
            let json = serde_json::to_string(&f).unwrap();
            assert_eq!(f, serde_json::from_str::<Form>(&json).unwrap());
            let toml = toml::to_string(&f).unwrap();
            assert_eq!(f, toml::from_str::<Form>(&toml).unwrap());
        }
    }
    #[test]
    fn bounds_types_and_conditions() {
        let mut f = form(ParamKind::Int { min: 0, max: 10, default: 1 });
        for value in [Value::Number(-1.0), Value::Number(11.0), Value::Number(0.5), Value::Number(f64::NAN), Value::Bool(true)] {
            assert!(f.validate_values(&Values::from([("value".into(), value)])).is_err())
        }
        assert!(f.validate_values(&Values::from([("unknown".into(), Value::Number(1.0))])).is_err());
        let cond = Cond::Eq { key: "value".into(), value: Value::Number(1.0) };
        let v = f.defaults();
        assert!(cond.eval(&v));
        assert!(!Cond::Not(Box::new(cond.clone())).eval(&v));
        assert!(Cond::Any(vec![cond.clone()]).eval(&v));
        assert!(Cond::All(vec![cond.clone()]).eval(&v));
        f.params[0].visible_when = Some(Cond::All(vec![cond; 257]));
        assert!(f.validate().is_err());
        f.params[0].visible_when = None;
        f.params = vec![f.params[0].clone(); 129];
        assert!(f.validate().is_err());
    }
    #[test]
    fn all_patch_ops_are_atomic() {
        let f = form(ParamKind::Number { min: None, max: None, default: 1.0 });
        let mut s = FormState { values: f.defaults(), ..Default::default() };
        let mut p = Patch::none(0).set("value", Value::Number(2.0)).disable("value").hide("value");
        f.apply_patch(&mut s, &p).unwrap();
        assert!(s.hidden.contains("value") && s.disabled.contains("value"));
        p = Patch::none(1).enable("value").show("value");
        p.ops.push(PatchOp::Reset { key: "value".into() });
        f.apply_patch(&mut s, &p).unwrap();
        assert_eq!(s.values, f.defaults());
        assert!(s.hidden.is_empty() && s.disabled.is_empty());
        let before = s.clone();
        assert!(f.apply_patch(&mut s, &Patch::none(2).hide("unknown")).is_err());
        assert_eq!(s, before);
    }
}
