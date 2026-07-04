mod client;
mod config;
mod error;
mod retry;

pub use client::{
    ACCEPT_GITHUB_V3, ACCEPT_JSON, get_text, get_text_with_accept, get_text_with_accept_and_retry,
    post_text,
};
pub use config::{HttpConfig, HttpConfigInput, HttpHeader, HttpHeaderInput};
pub use error::HttpError;
pub use retry::RetryPolicy;
