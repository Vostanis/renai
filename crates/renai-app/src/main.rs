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
        .mount("/js", FileServer::new(relative!("/js"), Options::Missing | Options::NormalizeDirs))

        // register routes
        .mount("/", routes![
            root,
            stocks
        ])
}

// home
#[get("/")]
async fn root() -> Template {
    Template::render("root", context! { 
        intro_header: "Introduction", 
        intro_para: "This is the intro."
    })
}

// stocks
#[get("/stocks")]
async fn stocks() -> Template {
    let labels = vec!["January", "February", "March", "April", "May", "June", "July"];
    let data = vec![10, 20, 30, 40, 50, 60, 70];

    Template::render("stocks", context! { 
        ticker: "NVDA",
        labels: labels.into_iter().map(|x| x.to_string()).collect::<Vec<String>>(),
        data: data
    })
}