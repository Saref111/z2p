use super::super::helpers::prepare_html_template;

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
    prepare_html_template(
        &[("name", name), ("link", link)],
        "confirm_subscription_letter.html",
    )
}
