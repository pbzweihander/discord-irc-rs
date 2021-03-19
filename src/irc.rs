use anyhow::Result;
use libirc::client::prelude::{Command, Message, Prefix};

use crate::config::IrcConfig;

pub async fn handle_irc(
    msg: Message,
    http: &serenity::http::client::Http,
    webhook_id: u64,
    webhook_token: String,
    config: IrcConfig,
) -> Result<()> {
    match msg.command {
        Command::ERROR(args) => error!("IRC> Error {}", args),
        Command::PRIVMSG(_, content) => {
            if let Some(Prefix::Nickname(nickname, _, _)) = msg.prefix {
                if config.ignores.contains(&nickname) {
                    debug!("IRC| <{}(ignored)> {}", nickname, content);
                } else {
                    info!("IRC> <{}> {}", nickname, content);

                    http.get_webhook_with_token(webhook_id, &webhook_token)
                        .await?
                        .execute(http, true, |builder| {
                            builder.username(nickname).content(content)
                        })
                        .await?;
                }
            }
        }
        _ => {
            debug!("IRC> {:?}", msg);
        }
    }

    Ok(())
}
