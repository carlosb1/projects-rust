use rocket::Request;
use rocket_contrib::templates::Template;
use tera::Context;

#[catch(404)]
pub fn not_found(req: &Request) -> Template {
    let context = Context::new();
    Template::render("errors/not-found", &context)
}

// https://api.rocket.rs/rocket_contrib/struct.Template.html
