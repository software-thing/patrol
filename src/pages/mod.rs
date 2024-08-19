use poem::{handler, web::Redirect, IntoResponse, Response};

pub mod account;
pub mod login;
pub mod logout;
pub mod register;

#[handler]
pub async fn index() -> Response {
    Redirect::see_other("/patrol/login").into_response()
}
