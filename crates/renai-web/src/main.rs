use actix_web::{middleware::Logger, web, App, HttpServer};
use deadpool_postgres::{Config, ManagerConfig, RecyclingMethod, Runtime};
use dotenv::{dotenv, var};
use tokio_postgres::NoTls;
use utoipa::OpenApi;

// need to pick favourite documentation; currently considering redoc vs rapidoc
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

mod api;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    dotenv().ok();
    env_logger::init();

    // build pool from .env DATABASE_URL
    let db_url = var("POSTGRES_URL").expect("POSTGRES_URL must be set");
    let mut cfg = Config::new();
    cfg.url = Some(db_url);
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    let pool = cfg
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("Failed to create pool");

    // create API documentation
    use api::*;
    #[derive(OpenApi)]
    #[openapi(paths(stock::index, stock::metrics))]
    struct ApiDoc;
    let openapi = ApiDoc::openapi();

    // run server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone()))
            // api endpoints
            .service(stock::index)
            .service(stock::metrics)
            .service(stock::prices)
            // api documentation
            .service(RapiDoc::with_openapi("/openapi.json", ApiDoc::openapi()).path("/rapidoc"))
            .service(Redoc::with_url("/redoc", ApiDoc::openapi())) // <-- crypto brokers use this
            .service(Scalar::with_url("/scalar", ApiDoc::openapi()))
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/openapi.json", openapi.clone()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
