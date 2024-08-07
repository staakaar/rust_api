use actix_web::{http::header::ContentType, HttpResponse};

pub async fn publish_newsletter_from() -> Result<HttpResponse, actix_web::Error> {
    let idempotency_key = uuid::Uuid::new_v4();
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"
        <!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta http-equiv="content-type" content="text/html"; charset=utf-8">
                    <title>Change password</title>
                </head>
                <body>
                    <form action="/admin/newsletters" method="post">
                        <label>Confirm new password
                            <input
                                type="password"
                                placeholder="Type the new password again"
                                name="new_password_check"
                            >
                        </label>
                        <br>
                        <input hidden type="text" name="idempotency_key" value="{idempotency_key}">
                        <button type="submit">Change password</button>
                    </form>>
                </body>
            </html>
        "#
        )))
}
