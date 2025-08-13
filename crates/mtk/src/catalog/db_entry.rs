use serde::Serialize;
use serde_json::json;

use crate::RepoPathBuf;

use super::SpecialEntryType;

// Retrieving values from JSON notes separated into user/external/generated.
fn get_notes_merged<T>(
    key: &str,
    user: Option<&serde_json::Value>,
    external: Option<&serde_json::Value>,
    generated: Option<&serde_json::Value>,
) -> Result<Option<T>, serde_json::Error>
where
    T: serde::de::DeserializeOwned,
{
    let merged_value = {
        let user_value = user.and_then(|v| v.get(key));
        if user_value.is_none() {
            let external_value = external.and_then(|v| v.get(key));
            if external_value.is_none() {
                let generated_value = generated.and_then(|v| v.get(key));
                generated_value
            } else {
                external_value
            }
        } else {
            user_value
        }
    };

    match merged_value {
        None => Ok(None),
        Some(v) => serde_json::from_value(v.clone()),
    }
}

// TODO: where to put this?
pub fn number_to_i64(n: &serde_json::Number) -> Option<i64> {
    if n.is_i64() {
        return n.as_i64();
    } else if n.is_f64() {
        n.as_f64().and_then(|f| {
            // Try to convert this f64 into an i64 losslessly
            // If there is any fractional part, or if the number is out of range, return None
            if (f as i64 as f64) == f {
                Some(f as i64)
            } else {
                None
            }
        })
    } else {
        return None;
    }
}

#[derive(Clone, Serialize)]
pub struct DbEntry {
    pub id: i64,
    pub repo_path: RepoPathBuf,
    pub deleted: bool,

    pub special_type: Option<SpecialEntryType>,
    pub associated_entry: Option<i32>,

    pub(crate) notes_user: serde_json::Value,
    pub(crate) notes_external: serde_json::Value,
    pub(crate) notes_generated: serde_json::Value,
    pub(crate) notes: serde_json::Value,
}

impl DbEntry {
    pub fn default(id: i64, repo_path: RepoPathBuf) -> DbEntry {
        DbEntry {
            id: id,
            repo_path: repo_path,
            deleted: false,
            special_type: None,
            associated_entry: None,
            notes_user: json!({}),
            notes_external: json!({}),
            notes_generated: json!({}),
            notes: json!({}),
        }
    }

    pub fn get<T>(&self, key: &str) -> Result<Option<T>, serde_json::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        // TODO(fyhuang): just return the merged value directly
        get_notes_merged(
            key,
            Some(&self.notes_user),
            Some(&self.notes_external),
            Some(&self.notes_generated),
        )
    }

    pub fn get_user<T>(&self, key: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.notes_user.get(key).map(|v| {
            serde_json::from_value(v.clone()).unwrap()
        })
    }

    // TODO(fyhuang): implement this in Catalog instead?
    /*pub fn set_user<T>(&mut self, key: &str, value: T)
    where
        T: serde::Serialize,
    {
        self.info_user[key] = serde_json::to_value(value).unwrap();
    }*/

    pub fn all_notes_user_json(&self) -> &serde_json::Value {
        &self.notes_user
    }

    // Well-known fields
    pub fn title(&self) -> Option<String> {
        self.get("title").ok().flatten()
    }

    pub fn description(&self) -> String {
        self.get("description").ok().flatten().unwrap_or("".to_string())
    }

    pub fn rating(&self) -> Option<i64> {
        self.get::<serde_json::Number>("rating")
            .ok()
            .flatten()
            .and_then(|n| number_to_i64(&n))
    }

    pub fn linked_urls(&self) -> Vec<String> {
        self.get("linked_urls").ok().flatten().unwrap_or(vec![])
    }
}

#[cfg(test)]
mod get_notes_merged_tests {
    use super::*;

    #[test]
    fn test_get_none() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            None,
            get_notes_merged::<String>("title", None, None, None)?
        );

        assert_eq!(
            None,
            get_notes_merged::<i64>("rating", None, None, None)?
        );

        Ok(())
    }

    #[test]
    fn test_get_simple() -> Result<(), Box<dyn std::error::Error>> {
        let user = serde_json::json!({
            "title": "user title",
        });
        assert_eq!(
            Some("user title".to_string()),
            get_notes_merged::<String>("title", Some(&user), None, None)?
        );
        assert_eq!(
            None,
            get_notes_merged::<String>("description", Some(&user), None, None)?
        );

        Ok(())
    }

    #[test]
    fn test_get_override_simple() -> Result<(), Box<dyn std::error::Error>> {
        let user = serde_json::json!({
            "title": "user title",
        });
        let external = serde_json::json!({
            "title": "external title",
        });
        let generated = serde_json::json!({
            "title": "generated title",
        });
        // User overrides external
        assert_eq!(
            Some("user title".to_string()),
            get_notes_merged::<String>("title", Some(&user), Some(&external), None)?
        );
        // External overrides generated
        assert_eq!(
            Some("external title".to_string()),
            get_notes_merged::<String>("title", None, Some(&external), Some(&generated))?
        );
        // User overrides generated
        assert_eq!(
            Some("user title".to_string()),
            get_notes_merged::<String>("title", Some(&user), None, Some(&generated))?
        );

        Ok(())
    }

    #[test]
    fn test_get_override_missing() -> Result<(), Box<dyn std::error::Error>> {
        let user = serde_json::json!({
            "title": "user title",
        });
        let external = serde_json::json!({
            "description": "external description",
        });
        assert_eq!(
            Some("external description".to_string()),
            get_notes_merged::<String>("description", Some(&user), Some(&external), None)?
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_to_i64() {
        assert_eq!(Some(5), number_to_i64(&serde_json::json!(5).as_number().unwrap()));
        assert_eq!(Some(6), number_to_i64(&serde_json::json!(6 as u64).as_number().unwrap()));
        assert_eq!(Some(7), number_to_i64(&serde_json::json!(7.0).as_number().unwrap()));

        assert_eq!(None, number_to_i64(&serde_json::json!(7.5).as_number().unwrap()));
        assert_eq!(None, number_to_i64(&serde_json::json!(u64::MAX).as_number().unwrap()));
        // This is the next f64 after i64::MAX
        assert_eq!(None, number_to_i64(&serde_json::json!(9223372036854777856.0).as_number().unwrap()));
    }

    #[test]
    fn test_title() {
        let mut row = DbEntry::default(0, RepoPathBuf::from("test"));
        assert!(row.title().is_none());

        row.notes_user = serde_json::json!({
            "title": "user title",
        });
        assert_eq!("user title", row.title().unwrap());
    }

    #[test]
    fn test_rating() {
        let mut row = DbEntry::default(0, RepoPathBuf::from("test"));
        assert_eq!(None, row.rating());

        row.notes_user = serde_json::json!({
            "rating": "not a number",
        });
        assert_eq!(None, row.rating());

        row.notes_user = serde_json::json!({
            "rating": 5,
        });
        assert_eq!(Some(5), row.rating());

        row.notes_user = serde_json::json!({
            "rating": 3.0,
        });
        assert_eq!(Some(3), row.rating());
    }
}
