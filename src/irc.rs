use anyhow::Result;
use libirc::client::prelude::{Command, Message, Prefix, Response};
use libirc::client::Sender;

use crate::config::{DiscordConfig, IrcConfig};
use crate::format::irc_msg_to_discord;

pub async fn handle_irc(
    msg: Message,
    irc_sender: Sender,
    discord: &serenity::CacheAndHttp,
    config: IrcConfig,
    discord_config: DiscordConfig,
) -> Result<()> {
    let DiscordConfig {
        channel_id,
        webhook_id,
        webhook_token,
        ..
    } = discord_config;
    match msg.command {
        Command::ERROR(args) => error!("IRC> Error {}", args),
        Command::Response(Response::RPL_WELCOME, _) => {
            if let Some(ozinger) = config.ozinger {
                irc_sender.send_oper(ozinger.username, ozinger.password)?;
            }

            irc_sender.send_join(&config.channel)?;
        }
        Command::PRIVMSG(_, content) => {
            if let Some(Prefix::Nickname(nickname, _, _)) = msg.prefix {
                if config.ignores.contains(&nickname) {
                    debug!("IRC| <{}(ignored)> {}", nickname, content);
                } else {
                    info!("IRC> <{}> {}", nickname, content);

                    let content = irc_msg_to_discord(&content);
                    discord
                        .http
                        .get_webhook_with_token(webhook_id, &webhook_token)
                        .await?
                        .execute(&discord.http, true, |builder| {
                            builder.username(nickname).content(content)
                        })
                        .await?;
                }
            }
        }
        Command::JOIN(..) => {
            if let Some(Prefix::Nickname(nickname, ..)) = msg.prefix {
                if config.bridge_member_changes
                    && config.connection.nickname.as_ref() == Some(&nickname)
                    && !config.ignores.contains(&nickname)
                {
                    serenity::model::id::ChannelId::from(channel_id)
                        .say(
                            &discord.http,
                            format_args!("**{}** has joined the channel.", nickname),
                        )
                        .await?;
                }
            }
        }
        Command::PART(_, comment) | Command::QUIT(comment) => {
            if let Some(Prefix::Nickname(nickname, ..)) = msg.prefix {
                if config.bridge_member_changes
                    && config.connection.nickname.as_ref() == Some(&nickname)
                    && !config.ignores.contains(&nickname)
                {
                    let mut message = format!("**{}** has left the channel.", nickname);
                    if let Some(comment) = comment {
                        message.push_str(" (`");
                        message.push_str(&comment);
                        message.push_str("`)");
                    }
                    serenity::model::id::ChannelId::from(channel_id)
                        .say(&discord.http, message)
                        .await?;
                }
            }
        }
        Command::KICK(_, nickname, comment) => {
            if let Some(Prefix::Nickname(kicked_by, ..)) = msg.prefix {
                if config.bridge_member_changes
                    && config.connection.nickname.as_ref() == Some(&nickname)
                    && !config.ignores.contains(&nickname)
                {
                    let mut message =
                        format!("**{}** has been kicked by **{}**.", nickname, kicked_by);
                    if let Some(comment) = comment {
                        message.push_str(" (`");
                        message.push_str(&comment);
                        message.push_str("`)");
                    }
                    serenity::model::id::ChannelId::from(channel_id)
                        .say(&discord.http, message)
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
