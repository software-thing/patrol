use poem::{
    handler,
    web::{Data, Html},
};
use tera::{Context, Tera};

use crate::{token::Claims, BASE_PATH};

#[handler]
pub async fn get(
    Data((tera, context)): Data<&(Tera, Context)>,
    Data(user): Data<&Claims>,
) -> anyhow::Result<Html<String>> {
    let mut ctx = Context::new();
    ctx.insert("base_path", BASE_PATH);
    ctx.insert("user", user);

    ctx.extend(context.clone());

    tera.render("account.html.tera", &ctx)
        .map_err(anyhow::Error::new)
        .map(Html)
}
