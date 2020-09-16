//! Actix web r2d2 example
use std::io;

use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

/// Async request handler. Db pool is stored in application state.
async fn index(
    path: web::Path<String>,
    db: web::Data<Pool<SqliteConnectionManager>>,
) -> Result<HttpResponse, Error> {
    // execute sync code in threadpool
    let res = web::block(move || {
        let conn = db.get().unwrap();

        // create table users if needed
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                        id TEXT PRIMARY KEY, 
                        name TEXT NOT NULL
            )",
            rusqlite::params![],
        )
        .unwrap();

        let uuid = format!("{}", uuid::Uuid::new_v4());
        let name = path.into_inner();

        // println!("{} {}", uuid, name);

        conn.execute(
            "INSERT INTO users (id, name) VALUES ($1, $2)",
            rusqlite::params![&uuid, &name],
        )
        .unwrap();
        /*
        {
            Ok(_) => println!("Insert ok"),
            Err(e) => println!("{:?}", e),
        } */

        conn.query_row("SELECT name FROM users WHERE id=$1", &[&uuid], |row| {
            row.get::<_, String>(0)
        })
    })
    .await
    .map(|user| HttpResponse::Ok().json(user))
    .map_err(|_| HttpResponse::InternalServerError())?;
    Ok(res)
}

/// Sync request handler. Db pool is stored in application state.
async fn index2(
    path: web::Path<String>,
    db: web::Data<Pool<SqliteConnectionManager>>,
) -> Result<HttpResponse, Error> {
    let conn = db.get().unwrap();

    // create table users if needed
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
                        id TEXT PRIMARY KEY, 
                        name TEXT NOT NULL
            )",
        rusqlite::params![],
    )
    .unwrap();

    let uuid = format!("{}", uuid::Uuid::new_v4());
    let name = path.into_inner();

    // println!("{} {}", uuid, name);
    conn.execute(
        "INSERT INTO users (id, name) VALUES ($1, $2)",
        rusqlite::params![&uuid, &name],
    )
    .unwrap();

    let res = conn.query_row("SELECT name FROM users WHERE id=$1", &[&uuid], |row| {
        row.get::<_, String>(0)
    });

    let res = res
        .map(|user| HttpResponse::Ok().json(user))
        .map_err(|_| HttpResponse::InternalServerError())?;
    Ok(res)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    // r2d2 pool
    let manager = SqliteConnectionManager::file("test.db");
    let pool = r2d2::Pool::new(manager).unwrap();

    // start http server
    HttpServer::new(move || {
        App::new()
            .data(pool.clone()) // <- store db pool in app state
            .wrap(middleware::Logger::default())
            //.route("/{name}", web::get().to(index))
            .route("/{name2}", web::get().to(index2))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
