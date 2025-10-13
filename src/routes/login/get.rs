use actix_web::{HttpRequest, HttpResponse, cookie::Cookie, http::header::ContentType};
use tera::{self, Context as TeraContext};

pub async fn login_form(req: HttpRequest) -> HttpResponse {
    let error_string = match req.cookie("_flash") {
        None => "".into(),
        Some(s) => s.value().to_string(),
    };

    let mut ctx = TeraContext::new();
    ctx.insert("error", &error_string);
    let tera = tera::Tera::new("views/**/*").expect("Failed to initialize Tera templates");
    let page_string = tera
        .render("login.html", &ctx)
        .expect("Failed rendering login page.");

    let mut response = HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(page_string);
    response
        .add_removal_cookie(&Cookie::new("_flash", ""))
        .unwrap();
    response
}
