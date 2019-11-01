use {
    crate::{
        config::IrcConfig,
        webhook::{execute_webhook, WebhookBody},
        write_irc,
    },
    failure::Fallible,
    futures::{compat::*, prelude::*},
    tokio::net::TcpStream,
    yaircc::*,
};

pub async fn handle_irc(
    msg: Message,
    writer: Writer<Compat01As03<TcpStream>>,
    discord_webhook: String,
    config: IrcConfig,
) -> Fallible<()> {
    match msg.code {
        Code::Error => {
            let args = msg.args.join(" ");

            error!("IRC> Error {}", args);
        }
        Code::Ping => {
            let args = msg.args.join(" ");

            write_irc!(writer, "PONG {}\n", args);

            debug!("IRC> PONG to {}", args);
        }
        Code::RplWelcome => {
            write_irc!(writer, "JOIN {}\n", config.channel);

            info!("IRC> Joinning to {}...", config.channel);

            return Ok(());
        }
        Code::Join => {
            if let Some(Prefix::User(PrefixUser { nickname, .. })) = msg.prefix {
                info!("IRC> Joinned to {} as {}", msg.args[0], nickname);
            }
        }
        Code::Privmsg => {
            let content = &msg.args[1];
            if let Some(Prefix::User(PrefixUser { nickname, .. })) = msg.prefix {
                if config.ignores.contains(&nickname) {
                    debug!("IRC| <{}(ignored)> {}", nickname, content);
                } else {
                    info!("IRC> <{}> {}", nickname, content);

                    let body = WebhookBody {
                        content: content.to_string(),
                        username: nickname,
                    };
                    execute_webhook(&discord_webhook, &body).await?;
                }
            }
        }
        _ => {
            debug!("IRC> {:?}", msg);
        }
    }

    Ok(())
}

pub async fn send_irc(writer: Writer<Compat01As03<TcpStream>>, msg: String) -> Fallible<()> {
    writer.raw(msg).err_into().await
}
