mod errors {
    use actix_web::{HttpResponse, ResponseError};
    use deadpool_postgres::PoolError;
    use derive_more::{Display, From};

    #[derive(Display, From, Debug)]
    pub enum RestError {
        NotFound,
        PoolError(PoolError),
        PostgresError(tokio_postgres::Error)
    }

    impl std::error::Error for RestError {}

    impl ResponseError for RestError {
        fn error_response(&self) -> HttpResponse {
            match *self {
                RestError::NotFound => HttpResponse::NotFound().finish(),
                RestError::PoolError(ref err) => {
                    HttpResponse::InternalServerError().body(err.to_string())
                }
                _ => HttpResponse::InternalServerError().finish(),
            }
        }
    }
}

mod handlers {
    use actix_web::{get, post, web, Error, HttpResponse};
    use deadpool_postgres::Pool;
    use crate::errors::RestError;

    #[get("/")]
    async fn status(db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
        let client = db_pool.get().await.map_err(RestError::PoolError)?;
        let rows = client.query(
            "SELECT count(*) FROM users WHERE login != $1", 
            &[&"hello world"]).await.map_err(RestError::PostgresError)?;
        let users: i64 = rows[0].get(0);
        Ok(HttpResponse::Ok().body(format!(
            "{} users", users
            )))
    }

    #[post("/thanks")]
    async fn thanks4(body: String, db_pool: web::Data<Pool>) 
        -> Result<HttpResponse, Error> {
        let client = db_pool.get().await.map_err(RestError::PoolError)?;
        let rows = client.query(
            "SELECT count(*) FROM users WHERE login != $1", 
            &[&"hello world"]).await.map_err(RestError::PostgresError)?;
        let users: i64 = rows[0].get(0);
        Ok(HttpResponse::Ok().body(format!(
            "Thanks for {} from {} users", body, users
            )))
    }
}

use tokio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::fs::File;
use tokio_postgres::NoTls;
use deadpool_postgres::{Config, ManagerConfig, RecyclingMethod, Runtime};
use actix_web::{web, App, HttpServer};
use std::env;

fn env_or_default(key: &str, default: &str) -> String {
    match env::var(key) {
        Ok(val) => val,
        Err(_) => default.to_string()
    }
}

#[tokio::main] 
async fn main() -> std::io::Result<()> {
    let db_pass_path = env_or_default("DB_PASS_PATH", "/run/secrets/db_password");
    let f = File::open(db_pass_path).await?;
    let mut reader = BufReader::new(f);
    let mut db_password = String::new();
    reader.read_line(&mut db_password).await?;
    if db_password.is_empty() {
        return Ok(());
    }

    let mut cfg = Config::new();
        cfg.dbname = Some(env_or_default("DB_NAME", "postgres"));
        cfg.host = Some(env_or_default("DB_HOST", "db"));
        cfg.user = Some(env_or_default("DB_USER", "postgres"));
        cfg.password = Some(db_password);
        cfg.manager = Some(ManagerConfig { 
            recycling_method: RecyclingMethod::Fast 
        });

    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(handlers::status)
            .service(handlers::thanks4)
    }).bind(("0.0.0.0", 8080))
        .unwrap().run().await

}

