use actix_web::error::InternalError;
use actix_web::http::header::{ContentType, LOCATION};
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use hmac::{Hmac, Mac};
use secrecy::ExposeSecret;
use secrecy::Secret;
use sqlx::PgPool;

use crate::authentication::{validate_credentials, AuthError, Credentials};
use crate::routes::error_chain_fmt;
use crate::startup::HmacSecret;

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication field")]
    AuthError(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error),
}

#[tracing::instrument(skip(form, pool, secret), fields(username=tracing::field::Empty, user_id=tracing::field::Empty))]
pub async fn login(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    secret: web::Data<HmacSecret>,
) -> Result<HttpResponse, InternalError<LoginError>> {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .finish())
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };
            let query_string = format!("error={}", urlencoding::Encoded::new(e.to_string()));

            let hmac_tag = {
                let mut mac =
                    Hmac::<sha2::Sha256>::new_from_slice(secret.0.expose_secret().as_bytes())
                        .unwrap();
                mac.update(query_string.as_bytes());
                mac.finalize().into_bytes()
            };

            let response = HttpResponse::SeeOther()
                .insert_header((
                    LOCATION,
                    format!("/login?{}&tag={:x}", query_string, hmac_tag),
                ))
                .finish();
            Err(InternalError::from_response(e, response))
        }
    }
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for LoginError {
    // fn status_code(&self) -> StatusCode {
    //     match self {
    //         LoginError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    //         LoginError::AuthError(_) => StatusCode::UNAUTHORIZED,
    //     }
    // }

    // fn error_response(&self) -> HttpResponse {
    //     HttpResponse::build(self.status_code())
    //         .content_type(ContentType::html())
    //         .body(format!(
    //             r#"
    //                 <!DOCTYPE html>
    //                 <html lang="en">
    //                 <head>
    //                     <meta http-equiv="content-type" content="text/html; charset=utf-8">
    //                     <title>Login</title>
    //                 </head>
    //                 <body>
    //                     <p><i>{}</i></p>
    //                     <form action="/login" method="post">
    //                         <label>Username<input type="text" placeholder="Enter Username" name="username"></label>
    //                         <label><input type="password" placeholder="Enter Password" name="password"></label>
    //                         <button type="submit">Login</button>
    //                     </form>
    //                 </body>
    //             "#, self))
    // }

    fn error_response(&self) -> HttpResponse {
        let query_string = format!("error={}", urlencoding::Encoded::new(self.to_string()));

        let secret: &[u8] = todo!();
        let hmac_tag = {
            let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret).unwrap();
            mac.update(query_string.as_bytes());
            mac.finalize().into_bytes()
        };

        HttpResponse::build(self.status_code())
            .insert_header((LOCATION, format!("/login?{query_string}&tag={hmac_tag:x}")))
            .finish()
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::SEE_OTHER
    }
}
