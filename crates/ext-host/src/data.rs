use crate::api::{Code, OpError};
use serde_json::{Map, Value};

pub const MAX_KEYS: usize = 64;
pub const MAX_BYTES: usize = 256 * 1024;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PluginData(Map<String, Value>);

impl PluginData {
    pub fn from_json(value: Option<&Value>) -> Self {
        Self(value.and_then(Value::as_object).cloned().unwrap_or_default())
    }

    pub fn to_json(&self) -> Option<Value> {
        (!self.0.is_empty()).then(|| Value::Object(self.0.clone()))
    }

    pub fn get(&self, key: &str) -> Value {
        self.0.get(key).cloned().unwrap_or(Value::Null)
    }

    pub fn set(&mut self, key: &str, value: Value) -> Result<(), OpError> {
        if key.is_empty() || key.len() > 64 || !key.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_')) {
            return Err(OpError::invalid("keys are 1 to 64 letters, digits, '.', '-' or '_'"));
        }
        let mut next = self.0.clone();
        if value.is_null() {
            next.remove(key);
        } else {
            next.insert(key.to_owned(), value);
        }
        if next.len() > MAX_KEYS {
            return Err(OpError::new(Code::Budget, format!("a plugin keeps at most {MAX_KEYS} keys in one place")));
        }
        if serde_json::to_vec(&next).map_or(usize::MAX, |b| b.len()) > MAX_BYTES {
            return Err(OpError::new(Code::Budget, "a plugin keeps at most 256 KiB of data in one place"));
        }
        self.0 = next;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Document,
    Layer(u64),
    Node(u64),
}

impl Target {
    pub fn parse(args: &Value) -> Result<Self, OpError> {
        match args.get("target") {
            None | Some(Value::Null) => Ok(Self::Document),
            Some(t) => match (t.get("layer").and_then(Value::as_u64), t.get("node").and_then(Value::as_u64)) {
                (Some(id), None) => Ok(Self::Layer(id)),
                (None, Some(id)) => Ok(Self::Node(id)),
                _ => Err(OpError::invalid("target is {\"layer\": id} or {\"node\": id}")),
            },
        }
    }
}

pub fn handle(op: &str, args: &Value, data: &mut PluginData) -> Result<Value, OpError> {
    let key = args.get("key").and_then(Value::as_str).ok_or_else(|| OpError::invalid("key is required"))?;
    match op {
        "project.data.get" => Ok(data.get(key)),
        "project.data.set" => data.set(key, args.get("value").cloned().unwrap_or(Value::Null)).map(|()| Value::Null),
        _ => Err(OpError::new(Code::NotFound, format!("this app has no operation {op}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn data_round_trips_deletes_with_null_and_refuses_oversize() {
        let mut d = PluginData::default();
        handle("project.data.set", &json!({"key": "rig", "value": {"bones": 3}}), &mut d).unwrap();
        assert_eq!(handle("project.data.get", &json!({"key": "rig"}), &mut d).unwrap(), json!({"bones": 3}));
        let saved = d.to_json();
        assert_eq!(PluginData::from_json(saved.as_ref()), d);
        handle("project.data.set", &json!({"key": "rig", "value": null}), &mut d).unwrap();
        assert_eq!(d.to_json(), None);
        let big = "x".repeat(MAX_BYTES);
        assert_eq!(handle("project.data.set", &json!({"key": "big", "value": big}), &mut d).unwrap_err().code, Code::Budget);
        assert_eq!(handle("project.data.set", &json!({"key": "bad key", "value": 1}), &mut d).unwrap_err().code, Code::Invalid);
        assert_eq!(Target::parse(&json!({"target": {"layer": 4}})).unwrap(), Target::Layer(4));
        assert!(Target::parse(&json!({"target": {"layer": 4, "node": 5}})).is_err());
    }
}
