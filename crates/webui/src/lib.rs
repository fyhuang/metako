#[macro_use] extern crate rocket;

// Byte range support for raw files
mod range_limited_file;
pub mod raw_file_responder;

// Templates
pub mod askama_tpl;

// Handlers
pub mod files;
pub mod entry;
pub mod preview;

pub mod history;
pub mod edit;
pub mod query;
pub mod save;
