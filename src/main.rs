mod api;
mod csv_parser;
mod generators;

use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use clap::{Arg, ArgMatches, Command};
use dotenvy::dotenv;
use log::info;
use std::error::Error;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();

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

        info!(
            "Starting test_data_gen web server on http://localhost:{}",
            port
        );
        info!("Upload CSV files at http://localhost:{}/api/upload", port);

        run_server(port).await?;
    } else {
        let filename = matches.get_one::<String>("FILE").unwrap();
        run_cli(filename)?;
    }

    Ok(())
}

async fn run_server(port: &str) -> std::io::Result<()> {
    let bind_address = format!("127.0.0.1:{}", port);

    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new().wrap(cors).service(
            web::scope("/api")
                .route("/health", web::get().to(api::handlers::health_check))
                .route("/upload", web::post().to(api::handlers::upload_csv))
                .route(
                    "/generate",
                    web::post().to(api::handlers::generate_placeholder),
                ),
        )
    })
    .bind(&bind_address)?
    .run()
    .await
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
