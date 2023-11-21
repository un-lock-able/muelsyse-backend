use actix_web::{get, web, App, HttpResponse, HttpServer};
use clap::Parser;
use env_logger;
use log;
use rusqlite::Connection;
use serde_derive::Serialize;
use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, Mutex},
};
mod config_parser;

#[derive(Serialize)]
struct StatusRespond {
    count: i32,
}

struct CountInDatabase {
    id: u8,
    count: u32,
}

#[get("/count")]
async fn join_count(global_count: web::Data<Arc<Mutex<i32>>>) -> HttpResponse {
    log::info!("Returning the count");
    if let Ok(count) = global_count.lock() {
        return HttpResponse::Ok().json(StatusRespond { count: *count });
    }
    log::info!("Lock the count variable failed.");
    HttpResponse::Ok().json(StatusRespond { count: 0 })
}

#[get("/join")]
async fn new_join(global_count: web::Data<Arc<Mutex<i32>>>) -> HttpResponse {
    log::info!("Add one to the count");
    if let Ok(mut count) = global_count.lock() {
        *count += 1;
        return HttpResponse::Ok().json(StatusRespond { count: *count });
    }
    log::info!("Lock the count variable failed.");
    HttpResponse::Ok().json(StatusRespond { count: 0 })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let args = config_parser::CmdArgs::parse();

    let settings: config_parser::AppConfig =
        serde_json::from_reader(BufReader::new(File::open(args.config)?))?;

    // let db_connection = Connection::open(settings.db_name.0).unwrap_or_else(|err| {
    //     log::info!("Open database connection failed: {err}. Shutting down.");
    //     std::process::exit(0);
    // });

    // Prepare the database.
    // db_connection.execute("create table if not EXISTS count_save (id INTEGER primary key unique, total_count INTEGER not null);", ());

    let current_count = Arc::new(Mutex::new(0));

    log::info!(
        "Start server on {}:{}",
        settings.server.bind_address.0,
        settings.server.bind_port.0
    );

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(current_count.clone()))
            .service(new_join)
            .service(join_count)
    })
    .bind((settings.server.bind_address.0, settings.server.bind_port.0))?
    .run()
    .await
}
