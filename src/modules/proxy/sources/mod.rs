pub mod alias;
pub mod http;
pub mod local;
pub mod s3;

pub use alias::AliasSource;
pub use http::HttpFetcher;
pub use local::LocalSource;
pub use s3::S3Source;
