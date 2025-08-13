use mtk::filter_sort::FilterSortOptions;
use crate::filter_sort_web;

pub struct FilterSortRenderer {
    options: FilterSortOptions,
}

impl FilterSortRenderer {
    pub fn new(options: FilterSortOptions) -> Self {
        Self { options }
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