use {
    failure::Fallible, futures::compat::*, lazy_static::lazy_static, reqwest::r#async::Client,
    serde::Serialize,
};

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

#[derive(Serialize)]
pub struct WebhookBody {
    pub content: String,
    pub username: String,
}

pub async fn execute_webhook(url: &str, body: &WebhookBody) -> Fallible<()> {
    CLIENT
        .post(url)
        .json(body)
        .send()
        .compat()
        .await?
        .error_for_status()
        .map(|_| ())
        .map_err(Into::into)
}
