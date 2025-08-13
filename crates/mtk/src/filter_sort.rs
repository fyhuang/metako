use serde::{Deserialize, Serialize};
use crate::Entry;

fn is_default<T>(value: &T) -> bool
where
    T: PartialEq + Default,
{
    *value == T::default()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SortBy {
    Name,
    ModTime,
    Size,
    Rating,
    FileType,
}

impl Default for SortBy {
    fn default() -> Self {
        SortBy::Name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::Asc
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct FilterSortOptions {
    // Sort options
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub sort_by: SortBy,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub sort_order: SortOrder,

    // Filter options
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_file_types: Option<Vec<String>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_rating: Option<(i64, i64)>,

    // TODO: filter by user notes
}

impl FilterSortOptions {
    pub fn sort_by_name(&self) -> &str {
        match self.sort_by {
            SortBy::Name => "Name",
            SortBy::ModTime => "ModTime", 
            SortBy::Size => "Size",
            SortBy::Rating => "Rating",
            SortBy::FileType => "FileType",
        }
    }
    
    pub fn sort_order_name(&self) -> &str {
        match self.sort_order {
            SortOrder::Asc => "Asc",
            SortOrder::Desc => "Desc",
        }
    }
    
    pub fn with_sort(&self, sort_by_str: &str) -> Self {
        let mut new_options = self.clone();
        
        // Parse the sort_by string
        new_options.sort_by = match sort_by_str {
            "Name" => SortBy::Name,
            "ModTime" => SortBy::ModTime,
            "Size" => SortBy::Size,
            "Rating" => SortBy::Rating,
            "File Type" => SortBy::FileType,
            _ => SortBy::Name,
        };
        
        // If we're clicking the same sort field, toggle the order
        if new_options.sort_by == self.sort_by {
            new_options.sort_order = match self.sort_order {
                SortOrder::Asc => SortOrder::Desc,
                SortOrder::Desc => SortOrder::Asc,
            };
        } else {
            new_options.sort_order = SortOrder::Asc;
        }
        
        new_options
    }

    /// Apply filtering to a list of entries
    pub fn filter_entries(&self, entries: &[Entry]) -> Vec<Entry> {
        entries.iter().filter(|entry| self.matches_filter(entry)).cloned().collect()
    }

    /// Check if a single entry matches the filter criteria
    fn matches_filter(&self, entry: &Entry) -> bool {
        // File type filter
        if let Some(ref file_types) = self.filter_file_types {
            let entry_type = if entry.fs.file_type.is_dir {
                "directory"
            } else if crate::filetype::is_video(&entry.fs.file_path) {
                "video"
            } else if crate::filetype::is_image(&entry.fs.file_path) {
                "image"
            } else {
                "other"
            };
            
            if !file_types.contains(&entry_type.to_string()) {
                return false;
            }
        }

        // Rating range filter
        if let Some((min_rating, max_rating)) = self.filter_rating {
            if let Some(rating) = entry.db.rating() {
                if rating < min_rating || rating > max_rating {
                    return false;
                }
            } else {
                // If no rating and we're filtering by rating, exclude
                return false;
            }
        }

        true
    }

    /// Sort a list of entries according to the sort options
    pub fn sort_entries(&self, entries: &mut [Entry]) {
        entries.sort_by(|a, b| {
            // First, sort folders before files
            let a_is_dir = a.fs.file_type.is_dir;
            let b_is_dir = b.fs.file_type.is_dir;
            
            if a_is_dir && !b_is_dir {
                return std::cmp::Ordering::Less;
            }
            if !a_is_dir && b_is_dir {
                return std::cmp::Ordering::Greater;
            }

            // Then sort by the specified criteria
            let ordering = match self.sort_by {
                // TODO: sort should be case-insensitive
                // TODO: name should sort by title if available, otherwise filename
                SortBy::Name => a.fs.file_name.cmp(&b.fs.file_name),
                SortBy::ModTime => a.fs.mod_time.cmp(&b.fs.mod_time),
                SortBy::Size => a.fs.size_bytes.cmp(&b.fs.size_bytes),
                SortBy::Rating => {
                    let a_rating = a.db.rating().unwrap_or(0);
                    let b_rating = b.db.rating().unwrap_or(0);
                    a_rating.cmp(&b_rating)
                },
                SortBy::FileType => {
                    let a_type = get_file_type_sort_key(a);
                    let b_type = get_file_type_sort_key(b);
                    a_type.cmp(&b_type)
                }
            };

            match self.sort_order {
                SortOrder::Asc => ordering,
                SortOrder::Desc => ordering.reverse(),
            }
        });
    }
}

fn get_file_type_sort_key(entry: &Entry) -> u8 {
    if entry.fs.file_type.is_dir {
        0
    } else if crate::filetype::is_video(&entry.fs.file_path) {
        1
    } else if crate::filetype::is_image(&entry.fs.file_path) {
        2
    } else {
        3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let options = FilterSortOptions::default();
        assert_eq!(options.sort_by, SortBy::Name);
        assert_eq!(options.sort_order, SortOrder::Asc);
    }

    #[test]
    fn test_with_sort_toggle_same_field() {
        let options = FilterSortOptions {
            sort_by: SortBy::Name,
            sort_order: SortOrder::Asc,
            ..Default::default()
        };
        
        let new_options = options.with_sort("Name");
        
        // Should toggle to descending
        assert_eq!(new_options.sort_by, SortBy::Name);
        assert_eq!(new_options.sort_order, SortOrder::Desc);
    }

    #[test]
    fn test_with_sort_different_field() {
        let options = FilterSortOptions {
            sort_by: SortBy::Name,
            sort_order: SortOrder::Desc,
            ..Default::default()
        };
        
        let new_options = options.with_sort("Size");
        
        // Should switch to Size and reset to Asc
        assert_eq!(new_options.sort_by, SortBy::Size);
        assert_eq!(new_options.sort_order, SortOrder::Asc);
    }

    #[test]
    fn test_compact_serialization() {
        // Test that default values are omitted in serialization
        let default_options = FilterSortOptions::default();
        let json = serde_json::to_string(&default_options).unwrap();
        
        // Should be empty object since all fields have default values
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_sort_by_name_methods() {
        let options = FilterSortOptions {
            sort_by: SortBy::ModTime,
            sort_order: SortOrder::Desc,
            ..Default::default()
        };
        
        assert_eq!(options.sort_by_name(), "ModTime");
        assert_eq!(options.sort_order_name(), "Desc");
    }
}
