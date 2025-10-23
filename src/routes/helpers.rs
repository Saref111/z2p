use std::error::Error;

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
    for (key, value) in entries.to_vec() {
        ctx.insert(key, value);
    }
    let tera = tera::Tera::new("views/**/*").expect("Failed to initialize Tera templates");
    tera.render(template_name, &ctx)
        .expect("Failed rendering email template")
}
