//! Helpers for editing entries.

use serde_json::json;

pub fn increment_field_i64(
    info: &mut serde_json::Value,
    field_name: &str,
    increment: i64,
) -> Result<(), String> {
    let curr_i64 = {
        let field = &info[&field_name];
        if field.is_null() {
            0
        } else if field.is_number() {
            let curr_number = field.as_number().expect("as_number");
            super::db_entry::number_to_i64(curr_number).expect("number_to_i64")
        } else {
            return Err(format!("Field {} is not a number", field_name));
        }
    };

    info[&field_name] = json!(curr_i64 + increment);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment_field_i64() {
        // Normal case
        let mut info = serde_json::json!({
            "field": 5,
        });

        increment_field_i64(&mut info, "field", 3).unwrap();
        assert_eq!(8, info["field"].as_i64().unwrap());

        increment_field_i64(&mut info, "field", -2).unwrap();
        assert_eq!(6, info["field"].as_i64().unwrap());

        increment_field_i64(&mut info, "field", -6).unwrap();
        assert_eq!(0, info["field"].as_i64().unwrap());

        // Test with a null field
        increment_field_i64(&mut info, "field2", 3).unwrap();
        assert_eq!(3, info["field2"].as_i64().unwrap());

        // Test with a non-number field
        info["field3"] = json!("not a number");
        assert!(
            increment_field_i64(&mut info, "field3", 3).is_err()
        );
    }
}