mod errors {
    use actix_web::{HttpResponse, ResponseError};
    use deadpool_postgres::PoolError;
    use derive_more::{Display, From};

    #[derive(Display, From, Debug)]
    pub enum RestError {
        NotFound,
        InsufficientFunds,
        PoolError(PoolError),
        PostgresError(tokio_postgres::Error)
    }

    impl std::error::Error for RestError {}

    impl ResponseError for RestError {
        fn error_response(&self) -> HttpResponse {
            match *self {
                RestError::NotFound => HttpResponse::NotFound().finish(),
                RestError::InsufficientFunds => HttpResponse::BadRequest().finish(),
                RestError::PoolError(ref err) => {
                    HttpResponse::InternalServerError().body(err.to_string())
                }
                RestError::PostgresError(ref err) => {
                    HttpResponse::InternalServerError().body(err.to_string())
                }
                //_ => HttpResponse::InternalServerError().finish()
            }
        }
    }
}

mod handlers {
    use log::debug;
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
        let mut client = db_pool.get().await.map_err(RestError::PoolError)?;

        let transaction = client.transaction().await.map_err(RestError::PostgresError)?;

        let amount: i32 = 100;
        // Check that the balance of the account is greater than the amount
        let rows = transaction.query(
            "SELECT CAST(balance * 10000 AS BIGINT) FROM users WHERE login = $1 FOR UPDATE", 
            &[&body]).await.map_err(RestError::PostgresError)?;
        if rows.len() < 1 {
            return Err(RestError::NotFound.into());
        }
        let row = &rows[0];
        let balance: i64 = row.get(0);
        debug!("{} balance {} - {} = {}", body, balance, amount, balance - amount as i64);

        if balance < amount as i64 {
            return Err(RestError::InsufficientFunds.into());
        }
    
        transaction.execute("UPDATE users SET balance = balance - $1::INT / 10000.0 WHERE login = $2", &[&amount, &body]).await.map_err(RestError::PostgresError)?;
    
        transaction.commit().await.map_err(RestError::PostgresError)?;
        Ok(HttpResponse::Ok().body(format!(
            "{} thanks to user {}", amount, body
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

use actix_web::middleware::Logger;
use env_logger::Env;

#[tokio::main] 
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

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
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(web::Data::new(pool.clone()))
            .service(handlers::status)
            .service(handlers::thanks4)
    }).bind(("0.0.0.0", 8080))
        .unwrap().run().await

}

