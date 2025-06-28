#[macro_use] extern crate rocket;

// Byte range support for raw files
mod range_limited_file;
pub mod raw_file_responder;

// Handlers
pub mod files;
