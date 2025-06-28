use tera::{self, Context as TeraContext};

pub fn get_email_text(name: &str, link: &str) -> String {
    format!(
        "
        🎉 Welcome, {name}!

        Thank you for subscribing!

        To start receiving updates, please confirm your subscription by clicking the link below:

        {link}

        If you did not request this subscription, you can safely ignore this email.
    "
    )
}

pub fn get_email_html(name: &str, link: &str) -> String {
    let mut ctx = TeraContext::new();
    ctx.insert("name", name);
    ctx.insert("link", link);
    let tera = tera::Tera::new("views/**/*").expect("Failed to initialize Tera templates");
    tera.render("confirm_subscription_letter.html", &ctx)
        .expect("Failed rendering email template")
}
