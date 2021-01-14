use {
    failure::{err_msg, Error, Fallible},
    serde::Serialize,
};

#[derive(Serialize)]
pub struct WebhookBody {
    pub content: String,
    pub username: String,
}

pub async fn execute_webhook(url: &str, body: &WebhookBody) -> Fallible<()> {
    let body = surf::Body::from_json(body)
        .map_err(|err| AsRef::<dyn std::error::Error + Send + Sync>::as_ref(&err))?;
    let resp = surf::post(url)
        .body(body)
        .await?;
    let status = resp.status();
    if !status.is_success() {
        Err(err_msg(status.as_str().to_string()))
    } else {
        Ok(())
    }
}
