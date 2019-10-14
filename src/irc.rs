use {
    crate::{
        config::IrcConfig,
        webhook::{execute_webhook, WebhookBody},
    },
    failure::Fallible,
    futures::{compat::*, prelude::*},
    tokio::net::TcpStream,
    yaircc::*,
};

pub async fn handle_irc(
    config: IrcConfig,
    msg: Message,
    writer: Writer<Compat01As03<TcpStream>>,
    discord_webhook: String,
) -> Fallible<()> {
    match msg.code {
        Code::Error => {
            let args = msg.args.join(" ");

            error!("IRC> Error {}", args);

            return Ok(());
        }
        Code::Ping => {
            let args = msg.args.join(" ");

            writer.raw(format!("PONG {}\n", args)).await?;

            debug!("IRC> PONG to {}", args);

            return Ok(());
        }
        Code::RplWelcome => {
            writer.raw(format!("JOIN {}\n", config.channel)).await?;

            info!("IRC> Joinning to {}...", config.channel);

            return Ok(());
        }
        Code::Join => {
            if let Some(Prefix::User(PrefixUser { nickname, .. })) = msg.prefix {
                info!("IRC> Joinned to {} as {}", msg.args[0], nickname);

                return Ok(());
            }
        }
        Code::Privmsg => {
            let content = &msg.args[1];
            if let Some(Prefix::User(PrefixUser { nickname, .. })) = msg.prefix {
                info!("IRC> <{}> {}", nickname, content);

                let body = WebhookBody {
                    content: content.to_string(),
                    username: nickname,
                };
                execute_webhook(discord_webhook, body).await?;

                return Ok(());
            }
        }
        _ => (),
    }

    debug!("IRC> {:?}", msg);

    Ok(())
}

pub async fn send_irc(writer: Writer<Compat01As03<TcpStream>>, msg: String) -> Fallible<()> {
    writer.raw(msg).err_into().await
}
