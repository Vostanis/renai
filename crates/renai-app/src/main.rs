use rocket::{get, launch, routes};
use rocket::fs::{FileServer, Options, relative};
use rocket_dyn_templates::{context, Template};

#[launch]
fn rocket() -> _ {
    rocket::build()
        // add templating system
        .attach(Template::fairing())
        // serve content from disk
        .mount("/public", FileServer::new(relative!("/public"), Options::Missing | Options::NormalizeDirs))
        // register routes
        .mount("/", routes![root])
}

#[get("/")]
async fn root() -> Template {
    Template::render("root", context! { message: "renai"})
}

