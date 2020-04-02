use {
    failure::{err_msg, Error, Fallible},
    serde::Serialize,
};

#[derive(Serialize)]
pub struct WebhookBody {
    pub content: String,
    pub username: String,
}

#[derive(Serialize)]
pub struct OzingerWebhookBody {
    pub token: String,
    pub sender: String,
    pub target: String,
    pub message: String,
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

pub async fn execute_webhook_ozinger(body: &OzingerWebhookBody) -> Fallible<()> {
    let resp = surf::post("http://api.ozinger.org/chat")
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
