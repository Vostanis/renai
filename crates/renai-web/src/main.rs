use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use deadpool_postgres::{Client, Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use dotenv::{dotenv, var};
use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Deserialize, Serialize)]
struct StockIndex {
    ticker: String,
    title: String,
    industry: String,
}

#[get("stock/index")]
async fn stock_index(db_pool: web::Data<Pool>) -> impl Responder {
    // establish connection from pool
    let conn: Client = db_pool.get().await.expect("get connection from pool");

    // query the database
    let query = "
    SELECT
        ticker,
        title,
        industry
    FROM stock.index";
    let rows = match conn.query(query, &[]).await {
        Ok(rows) => rows,
        Err(e) => {
            println!("{e}");
            return HttpResponse::InternalServerError().body("Query execution failed");
        }
    };

    let data: Vec<StockIndex> = rows
        .iter()
        .map(|row| StockIndex {
            ticker: row.get("ticker"),
            title: row.get("title"),
            industry: row.get("industry"),
        })
        .collect();

    HttpResponse::Ok().json(data)
}

#[derive(Deserialize, Serialize)]
struct StockMetrics {
    date: String,
    metric: String,
    value: f64,
}

#[get("stock/metrics/{ticker}")]
async fn stock_metrics(path: web::Path<String>, db_pool: web::Data<Pool>) -> impl Responder {
    // establish connection from pool
    let conn: Client = db_pool.get().await.expect("get connection from pool");

    // query the database
    let ticker = path.into_inner();
    let query = "
    SELECT
        dated::VARCHAR,
        metric,
        val
    FROM q.stock_metrics WHERE ticker = $1";
    let rows = match conn.query(query, &[&ticker]).await {
        Ok(rows) => rows,
        Err(e) => {
            println!("{e}");
            return HttpResponse::InternalServerError().body("Query execution failed");
        }
    };

    let data: Vec<StockMetrics> = rows
        .iter()
        .map(|row| StockMetrics {
            date: row.get("dated"),
            metric: row.get("metric"),
            value: row.get("val"),
        })
        .collect();

    HttpResponse::Ok().json(data)
}

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

    // run server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
            .service(stock_index)
            .service(stock_metrics)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
