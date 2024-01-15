use std::env;

use dotenv::dotenv;
use poem::{get, listener::TcpListener, middleware::CookieJarManager, EndpointExt, Route, Server};
use poem_openapi::OpenApiService;
use sea_orm::Database;
use tera::Tera;

use crate::is_first_admin_registered::is_first_admin_registered;

mod api;
mod crypto;
mod is_first_admin_registered;
mod keys;
mod models;
mod pages;
mod token;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;

    // Setup templating
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

    // Connect to the database
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let database = Database::connect(&database_url).await?;

    let is_first_admin_registered = is_first_admin_registered(&database).await?;

    let api = OpenApiService::new(api::jwks::Api, "Patrol", env!("CARGO_PKG_VERSION"));

    let authenticated_routes = Route::new()
        .at("/account", get(pages::account::get))
        .around(token::token_middleware);

    let app = Route::new()
        .nest("/_", api.clone())
        .at("/_/openapi.json", api.spec_endpoint())
        .at(
            "/register",
            get(pages::register::get).post(pages::register::post),
        )
        .at("/login", get(pages::login::get).post(pages::login::post))
        .nest("/", authenticated_routes)
        .with(CookieJarManager::new())
        .data(tera)
        .data(database)
        .data(is_first_admin_registered);

    Server::new(TcpListener::bind(("0.0.0.0", 8000)))
        .run(app)
        .await?;

    Ok(())
}
