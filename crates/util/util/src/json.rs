use chrono::{DateTime, FixedOffset};
use serde_json::Value;

/// Recursively walk a JSON value and replace RFC3339 timestamps ending with `Z`
/// to use the `+00:00` timezone offset.
pub fn json_replacer(mut value: Value) -> Value {
    fn replace(val: &mut Value) {
        match val {
            Value::String(s) => {
                if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                    let dt: DateTime<FixedOffset> = dt.with_timezone(dt.offset());
                    *s = dt.to_rfc3339().replace('Z', "+00:00");
                }
            }
            Value::Array(arr) => arr.iter_mut().for_each(replace),
            Value::Object(map) => {
                for (_k, v) in map.iter_mut() {
                    replace(v);
                }
            }
            _ => {}
        }
    }

    replace(&mut value);
    value
}
