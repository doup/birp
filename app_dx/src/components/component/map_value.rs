use client::{Value, json};

use crate::bevy_type;

pub fn map_value(bevy_type: &str, value: Value) -> Value {
    match bevy_type {
        bevy_type::GLAM_QUAT => {
            if let Some(arr) = value.as_array() {
                let x = arr.first().and_then(Value::as_f64).unwrap_or(0.0);
                let y = arr.get(1).and_then(Value::as_f64).unwrap_or(0.0);
                let z = arr.get(2).and_then(Value::as_f64).unwrap_or(0.0);
                let w = arr.get(3).and_then(Value::as_f64).unwrap_or(1.0);
                json!({ "x": x, "y": y, "z": z, "w": w })
            } else {
                value
            }
        }
        bevy_type::GLAM_VEC2 => {
            if let Some(arr) = value.as_array() {
                let x = arr.first().and_then(Value::as_f64).unwrap_or(0.0);
                let y = arr.get(1).and_then(Value::as_f64).unwrap_or(0.0);
                json!({ "x": x, "y": y })
            } else {
                value
            }
        }
        bevy_type::GLAM_VEC3 | bevy_type::GLAM_VEC3A => {
            if let Some(arr) = value.as_array() {
                let x = arr.first().and_then(Value::as_f64).unwrap_or(0.0);
                let y = arr.get(1).and_then(Value::as_f64).unwrap_or(0.0);
                let z = arr.get(2).and_then(Value::as_f64).unwrap_or(0.0);
                json!({ "x": x, "y": y, "z": z })
            } else {
                value
            }
        }
        _ => value,
    }
}
