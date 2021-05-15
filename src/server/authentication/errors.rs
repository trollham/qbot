use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    JwtValidation(#[from] jsonwebtoken::errors::Error),
    #[error("HTTP request failed")]
    Fetch(#[from] reqwest::Error),
    #[error("Unknown")]
    Unknown,
}

pub mod reject {
    use warp::reject::Reject;

    #[derive(Debug)]
    pub struct BadGateway;
    impl Reject for BadGateway {}

    #[derive(Debug)]
    pub struct Unauthorized;
    impl Reject for Unauthorized {}
}
