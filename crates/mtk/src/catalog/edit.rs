//! Helpers for editing entries.

use serde_json::json;

/// Increment/decrement a integer field in the JSON `notes` object by a specified amount.
/// The field must be a integer number or null/nonexistent.
pub fn increment_field_i64(
    notes: &mut serde_json::Value,
    field_name: &str,
    increment: i64,
) -> Result<(), String> {
    let curr_i64 = {
        let field = &notes[&field_name];
        if field.is_null() {
            0
        } else if field.is_number() {
            let curr_number = field.as_number().expect("as_number");
            super::db_entry::number_to_i64(curr_number).expect("number_to_i64")
        } else {
            return Err(format!("Field {} is not a number", field_name));
        }
    };

    notes[&field_name] = json!(curr_i64 + increment);
    Ok(())
}

/// A mini (only one level) version of RFC 7396 JSON Merge Patch
pub fn minimerge_json_value(
    base: &serde_json::Value,
    patch: &serde_json::Value,
) -> serde_json::Value {
    if !base.is_object() || !patch.is_object() {
        panic!("mergepatch_json_value: base and patch must be objects")
    }

    let patch_object = patch.as_object().unwrap();

    let mut merged_obj = base.clone();
    let merged = merged_obj.as_object_mut().unwrap();
    for (key, value) in patch_object {
        if value.is_null() {
            merged.remove(key);
        } else {
            merged[key] = value.clone();
        }
    }

    merged_obj
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment_field_i64() {
        // Normal case
        let mut info = serde_json::json!({
            "field": 5,
            "null_field": null,
        });

        increment_field_i64(&mut info, "field", 3).unwrap();
        assert_eq!(8, info["field"].as_i64().unwrap());

        increment_field_i64(&mut info, "field", -2).unwrap();
        assert_eq!(6, info["field"].as_i64().unwrap());

        increment_field_i64(&mut info, "field", -6).unwrap();
        assert_eq!(0, info["field"].as_i64().unwrap());

        // Test with a nonexistent field
        increment_field_i64(&mut info, "field2", 3).unwrap();
        assert_eq!(3, info["field2"].as_i64().unwrap());

        // Test with a null field
        increment_field_i64(&mut info, "null_field", 5).unwrap();
        assert_eq!(5, info["null_field"].as_i64().unwrap());

        // Test with a non-number field
        info["field3"] = json!("not a number");
        assert!(
            increment_field_i64(&mut info, "field3", 3).is_err()
        );
    }

    #[test]
    fn test_minimerge() {
        let result = minimerge_json_value(
            // Base
            &json!({
                "replace_number": 1,
                "replace_string": 2,
                "replace_object": 3,
                "delete_number": 4,
                "replace_list": [1, 2, 3],
                "delete_list": [1, 2, 3],
                "delete_object": {
                    "key": "value",
                },
                "replace_object_nomerge": {
                    "key": 1,
                },
                "untouched_number": 5,
                "untouched_string": "hello world",
            }),
            // Patch
            &json!({
                "replace_number": 2,
                "replace_string": "goodbye world",
                "replace_object": {
                    "new_key": "new_value"
                },
                "delete_number": null,
                "replace_list": [3, 4, 5],
                "delete_list": null,
                "delete_object": null,
                "replace_object_nomerge": {
                    "new_key": 2,
                },
            })
        );

        let result = result.as_object().unwrap();

        // Check that null deletes
        assert!(result.contains_key("delete_number") == false);
        assert!(result.contains_key("delete_list") == false);
        assert!(result.contains_key("delete_object") == false);

        // Check that values are replaced
        assert_eq!(result["replace_number"], json!(2));
        assert_eq!(result["replace_string"], json!("goodbye world"));
        assert_eq!(result["replace_object"], json!({
            "new_key": "new_value"
        }));
        assert_eq!(result["replace_list"], json!([3, 4, 5]));

        // Check that object replacement does not recurse
        assert_eq!(result["replace_object_nomerge"], json!({
            "new_key": 2,
        }));
        assert!(result["replace_object_nomerge"]["key"].is_null());

        // Check that other values untouched
        assert_eq!(result["untouched_number"], json!(5));
        assert_eq!(result["untouched_string"], json!("hello world"));
    }
}
