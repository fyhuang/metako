use rocket::http::{ContentType, Header};
use rocket::request::Request;
use rocket::response::{self, Responder, Response};

use crate::range_limited_file::RangeLimitedFile;

fn last_modified_header(mod_time: &chrono::DateTime<chrono::Utc>) -> Option<Header<'static>> {
    // See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Last-Modified>
    Some(Header::new(
        "Last-Modified",
        mod_time.format("%a, %d %b %Y %H:%M:%S GMT").to_string(),
    ))
}

pub struct RawFileResponder {
    pub file: std::fs::File,
    pub size_bytes: u64,
    pub mod_time: chrono::DateTime<chrono::Utc>,
    pub content_type: ContentType,
    pub cache_control: Option<String>,
}

impl RawFileResponder {
    fn respond_norange<'r>(self, req: &'r Request<'_>) -> response::Result<'static> {
        let mut builder = Response::build();
        builder
            .header(self.content_type)
            .header(Header::new("Accept-Ranges", "bytes"));
        if let Some(lmh) = last_modified_header(&self.mod_time) {
            builder.header(lmh);
        }
        if let Some(cc) = self.cache_control {
            builder.header(Header::new("Cache-Control", cc));
        }

        if req.method() == rocket::http::Method::Get {
            builder.sized_body(
                self.size_bytes as usize,
                tokio::fs::File::from(self.file),
            );
        }

        builder.ok()
    }

    fn respond_range<'r>(
        self,
        req: &'r Request<'_>,
        range_header: &str,
    ) -> response::Result<'static> {
        let mut builder = Response::build();
        builder
            .header(self.content_type)
            .header(Header::new("Accept-Ranges", "bytes"));
        if let Some(lmh) = last_modified_header(&self.mod_time) {
            builder.header(lmh);
        }
        if let Some(cc) = self.cache_control {
            builder.header(Header::new("Cache-Control", cc));
        }

        if req.method() == rocket::http::Method::Head {
            return builder.ok();
        }
        assert!(req.method() == rocket::http::Method::Get);

        // Parse the range header
        // TODO(fyhuang): support more than just the 1st range
        let parsed = http_range::HttpRange::parse(range_header, self.size_bytes).unwrap()[0];

        let tokio_file = tokio::fs::File::from(self.file);

        builder
            .status(rocket::http::Status::PartialContent)
            .header(Header::new(
                "Content-Range",
                format!(
                    "bytes {}-{}/{}",
                    parsed.start,
                    parsed.start + parsed.length - 1,
                    self.size_bytes
                ),
            ))
            .sized_body(
                parsed.length as usize,
                RangeLimitedFile::new(tokio_file, parsed.start, parsed.length),
            )
            .ok()
    }
}

impl<'r> Responder<'r, 'static> for RawFileResponder {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let range_header_opt = req.headers().get_one("Range");
        if let Some(range_header) = range_header_opt {
            self.respond_range(req, range_header)
        } else {
            self.respond_norange(req)
        }
    }
}
