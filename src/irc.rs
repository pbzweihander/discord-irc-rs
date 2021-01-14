use {
    crate::{config::IrcConfig, write_irc},
    async_std::net::TcpStream,
    failure::Fallible,
    futures::prelude::*,
    yaircc::*,
};

pub async fn handle_irc(
    http: std::sync::Arc<serenity::http::client::Http>,
    msg: Message,
    writer: Writer<TcpStream>,
    webhook_id: u64,
    webhook_token: String,
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
            if let Some(ozinger) = config.ozinger {
                write_irc!(writer, "OPER {}\n", ozinger.authline);
            }
            write_irc!(writer, "JOIN {}\n", config.channel);

            info!("IRC> Joinning to {}...", config.channel);

            return Ok(());
        }
        Code::RplYoureoper | Code::ErrNooperhost => {
            info!("Operator authentication result: {}", msg.args[1]);
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

pub async fn send_irc(writer: Writer<TcpStream>, msg: String) -> Fallible<()> {
    writer.raw(msg).err_into().await
}
