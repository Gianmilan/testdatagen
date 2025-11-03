mod api;
mod csv_parser;
mod db;
mod generators;
mod multipart;

use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use clap::{Arg, ArgMatches, Command};
use dotenvy::dotenv;
use log::{info, warn};
use sqlx::__rt::timeout;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{SqlitePool, migrate};
use std::error::Error;
use std::time::Duration;
use tokio::signal;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();

    let database_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");
    info!("Using database: {}", database_url);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    migrate!().run(&pool).await?;

    let matches: ArgMatches = Command::new("testdatagen")
        .version("0.0.1")
        .author("GS")
        .about("Read CSV files, and generate entries. Start as web server with --serve")
        .arg(
            Arg::new("FILE")
                .help("Sets the input CSV path to use (CLI mode)")
                .index(1),
        )
        .arg(
            Arg::new("serve")
                .long("serve")
                .short('s')
                .help("Run as web server")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .short('p')
                .help("Port to run the web server on")
                .default_value("8080"),
        )
        .get_matches();

    if matches.get_flag("serve") || matches.get_one::<String>("FILE").is_none() {
        let port = matches
            .get_one::<String>("port")
            .map(|s| s.as_str())
            .unwrap_or("8080");

        let max_connections_val = dotenvy::var("MAX_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5);

        info!("Connecting to SQLite database");
        let pool = SqlitePoolOptions::new()
            .max_connections(max_connections_val)
            .connect(&database_url)
            .await?;

        info!("Running migrations");
        sqlx::migrate!("./migrations").run(&pool).await?;

        info!(
            "Starting test_data_gen web server on http://localhost:{}",
            port
        );

        run_server(port, pool).await?;
    } else {
        let filename = matches.get_one::<String>("FILE").unwrap();
        run_cli(filename)?;
    }

    Ok(())
}

async fn run_server(port: &str, pool: SqlitePool) -> std::io::Result<()> {
    let bind_address = format!("127.0.0.1:{}", port);

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(cors)
            .service(
                web::scope("/api")
                    .route("/health", web::get().to(api::handlers::health_check))
                    .route("/upload", web::post().to(api::handlers::upload_csv))
                    .route(
                        "/extract-headers",
                        web::post().to(api::handlers::extract_headers),
                    )
                    .route(
                        "/generate",
                        web::post().to(api::handlers::generate_placeholder),
                    )
                    .route("/datasets", web::get().to(api::handlers::datasets::list))
                    .route("/datasets", web::post().to(api::handlers::datasets::save))
                    .route(
                        "/datasets/{id}",
                        web::get().to(api::handlers::datasets::get_one),
                    )
                    .route(
                        "/datasets/{id}",
                        web::put().to(api::handlers::datasets::update),
                    )
                    .route(
                        "/datasets/{id}",
                        web::delete().to(api::handlers::datasets::delete),
                    )
                    .route(
                        "/datasets/{id}/generate",
                        web::post().to(api::handlers::datasets::generate_from_dataset),
                    )
                    .route(
                        "/datasets/{id}/duplicate",
                        web::post().to(api::handlers::datasets::duplicate),
                    ),
            )
    })
    .bind(&bind_address)?
    .run();

    let server_handle = server.handle();
    tokio::spawn(server);

    signal::ctrl_c().await.expect("Failed to listen for Ctrl-C");
    info!("Shutting down gracefully...");

    match timeout(Duration::from_secs(10), server_handle.stop(true)).await {
        Ok(_) => info!("Server stopped gracefully"),
        Err(_) => {
            warn!("Graceful shutdown timed out, forcing shutdown");
            server_handle.stop(false).await;
        }
    }
    
    Ok(())
}

fn run_cli(filename: &str) -> Result<(), Box<dyn Error>> {
    println!("Reading CSV file: {}", filename);
    let csv_data = csv_parser::parse_csv_from_file(filename)?;

    println!("Headers: {:?}", csv_data.headers);
    println!("\nRows:");
    for (idx, row) in csv_data.rows.iter().enumerate() {
        println!("{}: {:?}", idx + 1, row);
    }

    println!("\nTotal rows: {}", csv_data.rows.len());
    Ok(())
}
