use std::{
    env,
    net::{Ipv4Addr, SocketAddrV4},
    time::Duration,
};

use dotenvy::dotenv;
use poem::{
    endpoint::{EmbeddedFileEndpoint, EmbeddedFilesEndpoint},
    get,
    listener::TcpListener,
    middleware::CookieJarManager,
    EndpointExt, Route, Server,
};
use sea_orm::Database;
use tera::Tera;
use tokio::signal::ctrl_c;

use crate::is_first_admin_registered::is_first_admin_registered;

mod crypto;
mod is_first_admin_registered;
mod keys;
mod models;
mod pages;
mod token;
mod well_known;

const BASE_PATH: &'static str = "/patrol";

// #[derive(RustEmbed)]
// #[folder = "static/"]
// #[include = "*.css"]
// struct Static;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;

    pretty_env_logger::init();

    // Embed styles
    // let styles_path = Static::iter().next().expect("No styles found");
    // println!("{}", styles_path);

    // Setup templating
    log::info!("Loading templates");
    let tera = match Tera::new("templates/**/*") {
        Ok(mut t) => {
            t.autoescape_on(vec![".html"]);
            t
        }
        Err(e) => {
            eprintln!("Template parsing error: {}", e);
            std::process::exit(1);
        }
    };

    let mut context = tera::Context::new();
    // context.insert("styles", &styles_path);

    // Connect to the database
    log::info!("Connecting to the database");
    let database_url_dbmate = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let database_url = database_url_dbmate
        .strip_suffix("?sslmode=disable")
        .expect("DATABASE_URL is not set (or does not end with `?sslmode=disable`)");
    let database = Database::connect(database_url).await?;

    let is_first_admin_registered = is_first_admin_registered(&database).await?;

    // Connect to Redis
    log::info!("Connecting to Redis for token storage");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL is not set");
    let redis = redis::Client::open(redis_url)?
        .get_connection_manager()
        .await?;

    let authenticated_routes = Route::new()
        .at("/", get(pages::index))
        .at("/account", get(pages::account::get))
        .at("/logout", get(pages::logout::get))
        .around(token::token_middleware);

    let well_known_routes = Route::new().at("/jwks.json", get(well_known::jwks));

    let app = Route::new()
        .nest("/.well-known", well_known_routes)
        .at(
            "/register",
            get(pages::register::get)
                .post(pages::register::post)
                .around(token::not_logged_in_middleware),
        )
        .at(
            "/register/is-available",
            get(pages::register::is_available::get),
        )
        .at(
            "/login",
            get(pages::login::get)
                .post(pages::login::post)
                .around(token::not_logged_in_middleware),
        )
        // .nest(
        //     "/static".to_string() + &styles_path,
        //     EmbeddedFileEndpoint::<Static>::new(&styles_path),
        // )
        .nest("/", authenticated_routes)
        .with(CookieJarManager::new())
        .data((tera, context))
        .data(database)
        .data(redis)
        .data(is_first_admin_registered);

    log::info!("Starting server");
    let socket_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 7287);
    Server::new(TcpListener::bind(socket_addr))
        .run_with_graceful_shutdown(
            app,
            async move { ctrl_c().await.unwrap_or(()) },
            Some(Duration::from_secs(1)),
        )
        .await?;

    Ok(())
}
