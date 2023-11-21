use actix_web::{get, web, App, HttpResponse, HttpServer};
use clap::Parser;
use env_logger;
use log;
use serde_derive::Serialize;
use sqlite::{ConnectionThreadSafe, State};
use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, Mutex},
};
mod config_parser;

#[derive(Serialize)]
struct StatusRespond {
    count: i64,
}

#[get("/count")]
async fn join_count(global_count: web::Data<Arc<Mutex<i64>>>) -> HttpResponse {
    log::info!("Returning the count");
    if let Ok(count) = global_count.lock() {
        return HttpResponse::Ok().json(StatusRespond { count: *count });
    }
    log::info!("Lock the count variable failed.");
    HttpResponse::Ok().json(StatusRespond { count: 0 })
}

#[get("/join")]
async fn new_join(
    global_count: web::Data<Arc<Mutex<i64>>>,
    db_conn: web::Data<Arc<Mutex<ConnectionThreadSafe>>>,
) -> HttpResponse {
    log::info!("Add one to the count");

    if let Ok(mut count) = global_count.lock() {
        if let Ok(db_conn) = db_conn.lock() {
            if let Ok(_) = db_conn.execute(format!(
                "UPDATE count_save SET total_count = {} WHERE id = 1;",
                *count + 1
            )) {
                *count += 1;
            }
        }
        return HttpResponse::Ok().json(StatusRespond { count: *count });
    }
    log::info!("Lock the count variable failed.");
    HttpResponse::Ok().json(StatusRespond { count: 0 })
}

fn initialize_database(db_connection: &ConnectionThreadSafe) -> Result<(), sqlite::Error> {
    log::info!("Initializing database.");
    db_connection.execute("DROP TABLE IF EXISTS count_save;")?;
    db_connection.execute("CREATE TABLE IF NOT EXISTS count_save (id INTEGER PRIMARY KEY UNIQUE, total_count INTEGER NOT null);")?;
    db_connection.execute("INSERT INTO count_save VALUES (1, 0);")?;
    return Ok(());
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let args = config_parser::CmdArgs::parse();

    let settings: config_parser::AppConfig =
        serde_json::from_reader(BufReader::new(File::open(args.config)?))?;

    let db_connection =
        sqlite::Connection::open_thread_safe(settings.db_name.0).unwrap_or_else(|err| {
            log::info!("Open database connection failed: {err}. Shutting down.");
            std::process::exit(0);
        });

    // Prepare the database.
    if args.init_database {
        if let Err(err_detail) = initialize_database(&db_connection) {
            log::info!("Initialize database failed: {err_detail}. Shutting down.");
            std::process::exit(0);
        };
    }

    let mut count: i64 = 0;

    let mut check_count_statement = db_connection
        .prepare("SELECT total_count FROM count_save WHERE id = ?;")
        .unwrap_or_else(|err_detail| {
            log::info!("Read initial count excution failed: {err_detail}. Shutting down.");
            std::process::exit(0);
        });

    check_count_statement.bind((1, 1)).unwrap_or_else(|err_detail| {
        log::info!("Do bind failed: {err_detail}. Shutting down.");
        std::process::exit(0);
    });

    while let Ok(State::Row) = check_count_statement.next() {
        count = check_count_statement
            .read::<i64, _>("total_count")
            .unwrap_or_else(|err_detail| {
                log::info!("Iterate count failed: {err_detail}. Shutting down.");
                std::process::exit(0);
            });
    }

    drop(check_count_statement);

    let current_count = Arc::new(Mutex::new(count));

    let db_connection = Arc::new(Mutex::new(db_connection));

    log::info!(
        "Start server on {}:{}",
        settings.server.bind_address.0,
        settings.server.bind_port.0
    );

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(current_count.clone()))
            .app_data(web::Data::new(db_connection.clone()))
            .service(new_join)
            .service(join_count)
    })
    .bind((settings.server.bind_address.0, settings.server.bind_port.0))?
    .run()
    .await
}
