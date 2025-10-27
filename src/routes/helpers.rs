use std::error::Error;

use actix_web::{HttpResponse, http::header::LOCATION};
use actix_web_flash_messages::{IncomingFlashMessages, Level};
use std::fmt::Write;

pub fn error_chain_fmt(e: &impl Error, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    writeln!(f, "{e}\n")?;
    let mut current = e.source();

    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{cause}")?;
        current = cause.source();
    }

    Ok(())
}

pub fn prepare_html_template(entries: &[(&str, &str)], template_name: &str) -> String {
    let mut ctx = tera::Context::new();
    for (key, value) in entries.iter().copied() {
        ctx.insert(key, value);
    }
    let tera = tera::Tera::new("views/**/*").expect("Failed to initialize Tera templates");
    tera.render(template_name, &ctx)
        .expect("Failed rendering email template")
}

pub fn e500<T>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorInternalServerError(e)
}

pub fn see_other(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish()
}

pub fn get_message(flash_messages: IncomingFlashMessages, level: Option<Level>) -> String {
    let mut error_string = String::new();
    for m in flash_messages
        .iter()
        .filter(|m| level.is_none_or(|l| m.level() == l))
    {
        writeln!(error_string, "{}", m.content()).unwrap();
    }
    error_string
}
