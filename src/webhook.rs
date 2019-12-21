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
    let resp = surf::post(url)
        .body_json(body)?
        .await
        .map_err(Error::from_boxed_compat)?;
    let status = resp.status();
    if !status.is_success() {
        Err(err_msg(status.as_str().to_string()))
    } else {
        Ok(())
    }
}
