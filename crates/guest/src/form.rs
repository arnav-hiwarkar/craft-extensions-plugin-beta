//! Form builders matching the framework's parameter naming.
use craft_ext_core::form as wire;
pub use craft_ext_core::form::{Choice, Cond, FORM_SCHEMA, Param, ParamKind, PatchOp, PointSpace, Unit, Value, Values, Word};
use std::ops::RangeInclusive;
#[derive(Clone, Debug, serde::Serialize)]
#[serde(transparent)]
/// A framework-style builder around the versioned wire form.
pub struct Form(pub wire::Form);
impl std::ops::Deref for Form {
    type Target = wire::Form;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<Form> for wire::Form {
    fn from(f: Form) -> Self {
        f.0
    }
}
impl Form {
    /// Starts a keyed form with its translated title.
    pub fn new(key: impl Into<String>, title: impl Into<Word>) -> Self {
        Self(wire::Form { schema: FORM_SCHEMA, key: key.into(), title: title.into(), params: vec![], preview: None })
    }
    fn push(mut self, key: impl Into<String>, label: impl Into<Word>, kind: ParamKind) -> Self {
        self.0.params.push(Param { key: key.into(), label: label.into(), kind, unit: None, visible_when: None, enabled_when: None, help: None });
        self
    }
    /// Adds a bounded floating point range row.
    pub fn slider(self, key: impl Into<String>, label: impl Into<Word>, range: RangeInclusive<f64>, default: f64) -> Self {
        self.push(key, label, ParamKind::Range { min: *range.start(), max: *range.end(), default, step: None })
    }
    /// Adds an integer row with inclusive bounds.
    pub fn int(self, key: impl Into<String>, label: impl Into<Word>, range: RangeInclusive<i64>, default: i64) -> Self {
        self.push(key, label, ParamKind::Int { min: *range.start(), max: *range.end(), default })
    }
    /// Adds an unbounded number row.
    pub fn number(self, key: impl Into<String>, label: impl Into<Word>, default: f64) -> Self {
        self.push(key, label, ParamKind::Number { min: None, max: None, default })
    }
    /// Adds a choice menu with stable option ids.
    pub fn choice(self, key: impl Into<String>, label: impl Into<Word>, options: &[(&str, Word)], default: &str) -> Self {
        self.push(
            key,
            label,
            ParamKind::Choice {
                options: options.iter().map(|(id, label)| Choice { id: (*id).into(), label: label.clone() }).collect(),
                default: default.into(),
            },
        )
    }
    /// Adds radio buttons with stable option ids.
    pub fn radio(self, key: impl Into<String>, label: impl Into<Word>, options: &[(&str, Word)], default: &str) -> Self {
        self.push(
            key,
            label,
            ParamKind::Radio {
                options: options.iter().map(|(id, label)| Choice { id: (*id).into(), label: label.clone() }).collect(),
                default: default.into(),
            },
        )
    }
    /// Adds a boolean row.
    pub fn toggle(self, key: impl Into<String>, label: impl Into<Word>, default: bool) -> Self {
        self.push(key, label, ParamKind::Bool { default })
    }
    /// Adds an angle in degrees.
    pub fn angle(self, key: impl Into<String>, label: impl Into<Word>, default: f64) -> Self {
        self.push(key, label, ParamKind::Angle { default })
    }
    /// Adds straight RGBA in the host working space; alpha controls opacity editing.
    pub fn color(self, key: impl Into<String>, label: impl Into<Word>, default: [f32; 4], alpha: bool) -> Self {
        self.push(key, label, ParamKind::Color { default, alpha })
    }
    /// Adds a point in the declared coordinate space.
    pub fn point(self, key: impl Into<String>, label: impl Into<Word>, default: [f64; 2], space: PointSpace) -> Self {
        self.push(key, label, ParamKind::Point { default, space })
    }
    /// Adds a text row, optionally allowing line breaks.
    pub fn text(self, key: impl Into<String>, label: impl Into<Word>, default: impl Into<String>, multiline: bool) -> Self {
        self.push(key, label, ParamKind::Text { default: default.into(), multiline })
    }
    /// Adds a section heading, then the rows returned by the closure.
    pub fn section(self, label: impl Into<Word>, rows: impl FnOnce(Self) -> Self) -> Self {
        let key = format!("section-{}", self.0.params.len());
        rows(self.push(key, label, ParamKind::Section))
    }
    /// Sets the last row’s displayed unit.
    pub fn unit(mut self, unit: Unit) -> Self {
        if let Some(p) = self.0.params.last_mut() {
            p.unit = Some(unit)
        }
        self
    }
    /// Sets the last row’s visibility condition.
    pub fn visible_when(mut self, cond: Cond) -> Self {
        if let Some(p) = self.0.params.last_mut() {
            p.visible_when = Some(cond)
        }
        self
    }
    /// Sets the last row’s input condition.
    pub fn enabled_when(mut self, cond: Cond) -> Self {
        if let Some(p) = self.0.params.last_mut() {
            p.enabled_when = Some(cond)
        }
        self
    }
    /// Enables preview with comparison.
    pub fn preview(mut self) -> Self {
        self.0.preview = Some(wire::Preview { enabled: true, compare: true });
        self
    }
}
/// Creates a translated word keyed by the author crate name and English text.
#[macro_export]
macro_rules! t {
    ($text:expr) => {{ $crate::Word { id: Some(format!("{}.{}", env!("CARGO_PKG_NAME"), $text)), english: ($text).into() } }};
}
/// Borrowed typed parameter access. Choices are stable string ids.
pub struct Params<'a>(pub &'a Values);
impl<'a> Params<'a> {
    /// Reads a typed value, refusing a missing key.
    pub fn value(&self, key: &str) -> Result<&'a Value, String> {
        self.0.get(key).ok_or_else(|| format!("missing parameter `{key}`"))
    }
    /// Reads a number.
    pub fn f(&self, key: &str) -> Result<f64, String> {
        match self.value(key)? {
            Value::Number(n) => Ok(*n),
            _ => Err(format!("{key} is not a number")),
        }
    }
    /// Reads an exact, safely representable integer.
    pub fn int(&self, key: &str) -> Result<i64, String> {
        let n = self.f(key)?;
        if n.is_finite() && n.fract() == 0.0 && (-9007199254740991.0..=9007199254740991.0).contains(&n) {
            Ok(n as i64)
        } else {
            Err(format!("{key} is not an integer"))
        }
    }
    /// Reads a boolean.
    pub fn b(&self, key: &str) -> Result<bool, String> {
        match self.value(key)? {
            Value::Bool(v) => Ok(*v),
            _ => Err(format!("{key} is not a bool")),
        }
    }
    /// Reads text.
    pub fn text(&self, key: &str) -> Result<&'a str, String> {
        match self.value(key)? {
            Value::String(v) => Ok(v),
            _ => Err(format!("{key} is not text")),
        }
    }
    /// Reads a stable choice id.
    pub fn choice(&self, key: &str) -> Result<&'a str, String> {
        self.text(key)
    }
    /// Reads straight RGBA.
    pub fn color(&self, key: &str) -> Result<[f32; 4], String> {
        match self.value(key)? {
            Value::Color(v) => Ok(*v),
            _ => Err(format!("{key} is not a color")),
        }
    }
    /// Reads a point.
    pub fn point(&self, key: &str) -> Result<[f64; 2], String> {
        match self.value(key)? {
            Value::Point(v) => Ok(*v),
            _ => Err(format!("{key} is not a point")),
        }
    }
}
pub use wire::Patch;
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
/// The host’s complete current values and revision for an update.
pub struct Update {
    /// Current form values.
    pub values: Values,
    /// Revision the returned patch must use as its base.
    pub rev: u64,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builder_and_accessors() {
        let f = Form::new("example", crate::t!("Example"))
            .section("Basic", |s| s.slider("amount", "Amount", 0.0..=1.0, 0.5).unit(Unit::Percent).toggle("on", "On", true))
            .choice("mode", "Mode", &[("normal", crate::t!("Normal"))], "normal")
            .preview();
        f.0.validate().unwrap();
        let v = f.0.defaults();
        let p = Params(&v);
        assert_eq!(p.f("amount"), Ok(0.5));
        assert_eq!(p.choice("mode"), Ok("normal"));
        assert!(p.b("amount").is_err());
        assert!(f.0.title.id.unwrap().starts_with("craft-guest."));
    }
}
