use mtk::filter_sort::{FilterSortOptions, SortBy, SortOrder};
use crate::filter_sort_web;

const SORT_BY_NAME: &str = "Name";
const SORT_BY_MOD_TIME: &str = "Modified";
const SORT_BY_SIZE: &str = "Size";
const SORT_BY_RATING: &str = "Rating";
const SORT_BY_FILE_TYPE: &str = "File Type";

pub struct SortItemRenderer {
    pub name: &'static str,
    pub current_order: Option<SortOrder>,
}

pub struct FilterSortRenderer {
    options: FilterSortOptions,
}

impl FilterSortRenderer {
    pub fn new(options: FilterSortOptions) -> Self {
        Self { options }
    }

    pub fn all_sort_items(&self) -> Vec<SortItemRenderer> {
        let mut result = Vec::new();
        if self.options.sort_by == SortBy::Name {
            result.push(SortItemRenderer {
                name: SORT_BY_NAME,
                current_order: Some(self.options.sort_order.clone()),
            });
        } else {
            result.push(SortItemRenderer {
                name: SORT_BY_NAME,
                current_order: None,
            });
        }

        if self.options.sort_by == SortBy::ModTime {
            result.push(SortItemRenderer {
                name: SORT_BY_MOD_TIME,
                current_order: Some(self.options.sort_order.clone()),
            });
        } else {
            result.push(SortItemRenderer {
                name: SORT_BY_MOD_TIME,
                current_order: None,
            });
        }

        if self.options.sort_by == SortBy::Size {
            result.push(SortItemRenderer {
                name: SORT_BY_SIZE,
                current_order: Some(self.options.sort_order.clone()),
            });
        } else {
            result.push(SortItemRenderer {
                name: SORT_BY_SIZE,
                current_order: None,
            });
        }

        if self.options.sort_by == SortBy::Rating {
            result.push(SortItemRenderer {
                name: SORT_BY_RATING,
                current_order: Some(self.options.sort_order.clone()),
            });
        } else {
            result.push(SortItemRenderer {
                name: SORT_BY_RATING,
                current_order: None,
            });
        }

        if self.options.sort_by == SortBy::FileType {
            result.push(SortItemRenderer {
                name: SORT_BY_FILE_TYPE,
                current_order: Some(self.options.sort_order.clone()),
            });
        } else {
            result.push(SortItemRenderer {
                name: SORT_BY_FILE_TYPE,
                current_order: None,
            });
        }

        result
    }

    pub fn sort_by_name(&self) -> &str {
        self.options.sort_by_name()
    }

    pub fn sort_order_name(&self) -> &str {
        self.options.sort_order_name()
    }

    pub fn sort_url(&self, sort_by_str: &str) -> String {
        filter_sort_web::sort_url(&self.options, sort_by_str)
    }
}
