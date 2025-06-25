use std::{
    env,
    net::{Ipv4Addr, SocketAddrV4},
    time::Duration,
};

use dotenvy::dotenv;
use poem::{
    get, handler, http::StatusCode, listener::TcpListener, middleware::CookieJarManager,
    EndpointExt, Route, Server,
};
use sea_orm::Database;
use tera::Tera;
use tokio::{join, signal::ctrl_c};

use crate::is_first_admin_registered::is_first_admin_registered;

mod crypto;
mod is_first_admin_registered;
mod keys;
mod models;
mod pages;
mod session;
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

    let context = tera::Context::new();

    // Connect to the database
    log::info!("Connecting to the database");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let database = Database::connect(database_url).await?;

    let is_first_admin_registered = is_first_admin_registered(&database).await?;

    // Connect to Redis
    log::info!("Connecting to Redis for token storage");

    let authenticated_routes = Route::new()
        .at("/", get(pages::index))
        .at("/account", get(pages::account::get))
        .at("/logout", get(pages::logout::get));

    let well_known_routes = Route::new().at("/jwks.json", get(well_known::jwks));

    let app = Route::new()
        .at("/heartbeat", get(heartbeat))
        .nest("/.well-known", well_known_routes)
        .at(
            "/register",
            get(pages::register::get).post(pages::register::post),
        )
        .at(
            "/register/is-available",
            get(pages::register::is_available::get),
        )
        .at("/login", get(pages::login::get).post(pages::login::post))
        // .nest(
        //     "/static".to_string() + &styles_path,
        //     EmbeddedFileEndpoint::<Static>::new(&styles_path),
        // )
        .nest("/", authenticated_routes)
        .with(CookieJarManager::new())
        .data((tera, context))
        .data(database.clone())
        .data(is_first_admin_registered);

    let app_session = Route::new()
        .at("/session", get(session::ep::get))
        .data(database);

    log::info!("Starting server");

    let socket_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 7287);
    let socket_addr_session = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 7288);

    let _ = join!(
        Server::new(TcpListener::bind(socket_addr)).run_with_graceful_shutdown(
            app,
            async move { ctrl_c().await.unwrap_or(()) },
            Some(Duration::from_secs(1)),
        ),
        Server::new(TcpListener::bind(socket_addr_session)).run_with_graceful_shutdown(
            app_session,
            async move { ctrl_c().await.unwrap_or(()) },
            Some(Duration::from_secs(1)),
        ),
    );

    Ok(())
}

#[handler]
async fn heartbeat() -> StatusCode {
    StatusCode::OK
}
