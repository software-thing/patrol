use poem::handler;

#[handler]
pub async fn get() -> &'static str {
    "Hello, world!"
}
