pub mod http;
pub mod local;
pub mod router;
pub mod s3;

pub use http::HttpFetcher;
pub use local::LocalSource;
pub use router::SourceRouter;
pub use s3::S3Source;
