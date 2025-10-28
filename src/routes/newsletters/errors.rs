use actix_web::{
    HttpResponse, ResponseError,
    http::{
        StatusCode,
        header::{self, HeaderValue},
    },
};

use super::super::helpers::error_chain_fmt;

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("Authentication error.")]
    AuthError(#[source] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match self {
            PublishError::UnexpectedError(_) => {
                HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
            }
            PublishError::AuthError(_) => {
                let mut resp = HttpResponse::new(StatusCode::UNAUTHORIZED);

                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();

                resp.headers_mut()
                    .insert(header::WWW_AUTHENTICATE, header_value);

                resp
            }
        }
    }
}
