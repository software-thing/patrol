use std::net::{Ipv4Addr, SocketAddrV4};

use dotenvy::dotenv;
use poem::{
    get, handler, http::StatusCode, listener::TcpListener, middleware::CookieJarManager, post,
    EndpointExt, Route, Server,
};
use sea_orm::Database;
use tera::Tera;

use crate::is_first_admin_registered::is_first_admin_registered;

mod crypto;
mod is_first_admin_registered;
mod keys;
mod models;
mod pages;
mod session;
mod well_known;

const BASE_PATH: &'static str = "/patrol";

pub fn internal_server_error(err: impl std::error::Error) -> poem::Error {
    log::error!("{:?}", err);
    poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
}

// #[derive(RustEmbed)]
// #[folder = "static/"]
// #[include = "*.css"]
// struct Static;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    // Embed styles
    // let styles_path = Static::iter().next().expect("No styles found");
    // println!("{}", styles_path);

    // Setup templating
    log::info!("Loading templates");
    let tera = Tera::new("templates/**/*")
        .map(|mut t| {
            t.autoescape_on(vec![".html"]);
            t
        })
        .unwrap();

    let mut context = tera::Context::new();
    context.insert("base_path", BASE_PATH);

    // Connect to the database
    log::info!("Connecting to the database");
    let database = Database::connect("sqlite:data/patrol.db").await?;

    let is_first_admin_registered = is_first_admin_registered(&database).await?;

    let authenticated_routes = Route::new()
        .at("/", get(pages::index))
        .at("/account", get(pages::account::get))
        .at(
            "/account/change-password",
            post(pages::account::change_password),
        )
        .around(session::session_middleware);

    let app = Route::new()
        .at("/heartbeat", get(heartbeat))
        .at(
            "/register",
            get(pages::register::get).post(pages::register::post),
        )
        .at(
            "/register/is-available",
            get(pages::register::is_available::get),
        )
        .at("/login", get(pages::login::get).post(pages::login::post))
        .at("/logout", get(pages::logout::get))
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

    let _ = tokio::try_join!(
        Server::new(TcpListener::bind(socket_addr)).run(app),
        Server::new(TcpListener::bind(socket_addr_session)).run(app_session),
    );

    Ok(())
}

#[handler]
async fn heartbeat() -> StatusCode {
    StatusCode::OK
}
