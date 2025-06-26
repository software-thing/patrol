use poem::{
    handler,
    web::{Data, Html},
};
use tera::{Context, Tera};

use crate::{internal_server_error, session::Session};

#[handler]
pub async fn get(
    Data(session): Data<&Session>,
    Data((tera, context)): Data<&(Tera, Context)>,
) -> poem::Result<Html<String>> {
    let mut ctx = Context::new();

    ctx.extend(context.clone());
    ctx.insert("user", session);

    tera.render("account.html.tera", &ctx)
        .map_err(internal_server_error)
        .map(Html)
}
