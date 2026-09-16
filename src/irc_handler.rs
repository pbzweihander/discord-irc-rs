use anyhow::Result;
use irc::client::Sender;
use irc::client::prelude::{Command, Message, Prefix, Response};
use serenity::builder::{Builder, ExecuteWebhook};
use tracing::{debug, error, info, warn};

use crate::app_config::{DiscordConfig, IrcConfig};
use crate::format::irc_msg_to_discord;

pub async fn handle_irc(
    msg: Message,
    irc_sender: Sender,
    discord_cache: &serenity::cache::Cache,
    discord_http: &serenity::http::Http,
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
                        avatar = auto_detect_avatar(discord_cache, channel_id, &nickname);
                    }

                    let content = irc_msg_to_discord(&content);
                    let mut builder = ExecuteWebhook::new().username(nickname).content(content);
                    if let Some(avatar) = avatar {
                        builder = builder.avatar_url(avatar);
                    }
                    builder
                        .execute(discord_http, (webhook_id.into(), &webhook_token, true))
                        .await?;
                }
            }
        }
        Command::JOIN(..) => {
            if let Some(Prefix::Nickname(nickname, ..)) = msg.prefix
                && config.bridge_member_changes
                && config.connection.nickname.as_ref() != Some(&nickname)
                && !config.ignores.contains(&nickname)
            {
                serenity::model::id::ChannelId::from(channel_id)
                    .say(
                        discord_http,
                        format!("**{}** has joined the channel.", nickname),
                    )
                    .await?;
            }
        }
        Command::PART(_, comment) | Command::QUIT(comment) => {
            if let Some(Prefix::Nickname(nickname, ..)) = msg.prefix
                && config.bridge_member_changes
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
                    .say(discord_http, message)
                    .await?;
            }
        }
        Command::KICK(_, nickname, comment) => {
            if let Some(Prefix::Nickname(kicked_by, ..)) = msg.prefix
                && config.bridge_member_changes
                && config.connection.nickname.as_ref() == Some(&nickname)
                && !config.ignores.contains(&nickname)
            {
                let mut message = format!("**{}** has been kicked by **{}**.", nickname, kicked_by);
                if let Some(comment) = comment {
                    message.push_str(" (`");
                    message.push_str(&comment);
                    message.push_str("`)");
                }
                serenity::model::id::ChannelId::from(channel_id)
                    .say(discord_http, message)
                    .await?;
            }
        }
        _ => {
            debug!("IRC> {:?}", msg);
        }
    }

    Ok(())
}

fn auto_detect_avatar(
    cache: &serenity::cache::Cache,
    channel_id: u64,
    nickname: &str,
) -> Option<String> {
    let channel = cache.guilds().into_iter().find_map(|guild_id| {
        cache
            .guild(guild_id)?
            .channels
            .get(&channel_id.into())
            .cloned()
    });
    match channel {
        None => {}
        Some(channel) => match channel.members(cache) {
            Err(_) => {}
            Ok(members) => {
                for member in members {
                    if member.display_name() == nickname {
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
