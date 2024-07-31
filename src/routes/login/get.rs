use crate::startup::HmacSecret;
use actix_web::{
    cookie::{time::Duration, Cookie},
    http::header::ContentType,
    web, HttpRequest, HttpResponse,
};
use actix_web_flash_messages::{IncomingFlashMessages, Level};
use hmac::{Hmac, Mac};
use secrecy::ExposeSecret;
use std::fmt::Write;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: String,
    tag: String,
}

impl QueryParams {
    fn verify(self, secret: &HmacSecret) -> Result<String, anyhow::Error> {
        let tag = hex::decode(self.tag)?;
        let query_string = format!("error={}", urlencoding::Encoded::new(&self.error));

        let mut mac =
            Hmac::<sha2::Sha256>::new_from_slice(secret.0.expose_secret().as_bytes()).unwrap();
        mac.update(query_string.as_bytes());
        mac.verify_slice(&tag)?;

        Ok(self.error)
    }
}

pub async fn login_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let mut error_html = String::new();
    for m in flash_messages.iter().filter(|m| m.level() == Level::Error) {
        writeln!(error_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }
    let mut response = HttpResponse::Ok()
        .content_type(ContentType::html())
        .cookie(Cookie::build("_flash", "").max_age(Duration::ZERO).finish())
        .body(format!(r#"
                    <!DOCTYPE html>
                    <html lang="en">
                    <head>
                        <meta http-equiv="content-type" content="text/html; charset=utf-8">
                        <title>Login</title>
                    </head>
                    <body>
                        <p><i>{error_html}</i></p>
                        <form action="/login" method="post">
                            <label>Username<input type="text" placeholder="Enter Username" name="username"></label>
                            <label><input type="password" placeholder="Enter Password" name="password"></label>
                            <button type="submit">Login</button>
                        </form>
                    </body>
                "#));

    response
        .add_removal_cookie(&Cookie::new("_flash", ""))
        .unwrap();
    response
}
