use anyhow::Result;
use libirc::client::Sender;
use libirc::client::prelude::{Command, Message, Prefix, Response};
use serenity::{builder::ExecuteWebhook, utils::hashmap_to_json_map};

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

                    let mut avatar = None;
                    if config.auto_detect_avatar {
                        avatar = auto_detect_avatar(&discord.cache, channel_id, &nickname).await;
                    }

                    let content = irc_msg_to_discord(&content);
                    let mut builder = ExecuteWebhook::default();
                    builder.username(nickname).content(content);
                    if let Some(avatar) = avatar {
                        builder.avatar_url(avatar);
                    }
                    let json = hashmap_to_json_map(builder.0);
                    discord
                        .http
                        .execute_webhook(webhook_id, &webhook_token, true, &json)
                        .await?;
                }
            }
        }
        Command::JOIN(..) => {
            if let Some(Prefix::Nickname(nickname, ..)) = msg.prefix {
                if config.bridge_member_changes
                    && config.connection.nickname.as_ref() != Some(&nickname)
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

async fn auto_detect_avatar(
    cache: &serenity::cache::Cache,
    channel_id: u64,
    nickname: &str,
) -> Option<String> {
    match cache.guild_channel(channel_id).await {
        None => {}
        Some(channel) => match channel.members(&cache).await {
            Err(_) => {}
            Ok(members) => {
                for member in members {
                    if *member.display_name() == nickname {
                        return Some(member.face());
                    }
                }
                return None;
            }
        },
    };

    warn!("Cache missed while it should never be missed");
    None
}
