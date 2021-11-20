use std::borrow::Cow;

use libirc::client::prelude::Command as IrcCommand;
use libirc::client::Sender;
use serenity::model::channel::Message;
use serenity::prelude::*;

use crate::config::*;
use crate::utils::{insert_zero_width_spaces_into_nickname, normalize_irc_nickname};

pub struct DiscordHandler {
    config: DiscordConfig,
    irc_config: IrcConfig,
    irc_sender: Sender,
}

impl DiscordHandler {
    pub fn new(config: DiscordConfig, irc_config: IrcConfig, irc_sender: Sender) -> Self {
        DiscordHandler {
            config,
            irc_config,
            irc_sender,
        }
    }
}

#[serenity::async_trait]
impl EventHandler for DiscordHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        if !msg.author.bot && msg.channel_id == self.config.channel_id {
            let Context { http, cache, .. } = ctx;

            let content = msg.content_safe(&cache).await;
            let id = msg.author.id.0;
            let name = msg.author_nick(&http).await.unwrap_or(msg.author.name);
            let display_name = if self.irc_config.prevent_noti_by_nicknames {
                Cow::Owned(insert_zero_width_spaces_into_nickname(&name))
            } else {
                Cow::Borrowed(&name)
            };

            let lines = content
                .split('\n')
                .map(|s| Cow::Borrowed(s))
                .chain(msg.attachments.into_iter().map(|at| at.url).map(Into::into));

            if self.config.ignores.contains(&name) {
                for line in lines {
                    debug!("DIS| <{}(ignored)> {}", name, line);
                }
            } else {
                let is_ozinger = self.irc_config.ozinger.is_some();
                let channel = &self.irc_config.channel;
                for line in lines {
                    info!("DIS> <{}> {}", name, line);
                    let command = if is_ozinger {
                        IrcCommand::Raw(
                            "FAKEMSG".to_string(),
                            vec![
                                format!(
                                    "{}＠d!{:x}@pbzweihander/discord-irc-rs",
                                    normalize_irc_nickname(&name),
                                    id
                                ),
                                channel.to_string(),
                                line.into_owned(),
                            ],
                        )
                    } else {
                        IrcCommand::PRIVMSG(
                            channel.to_string(),
                            format!("<{}> {}", display_name, line),
                        )
                    };
                    if let Err(e) = self.irc_sender.send(command) {
                        error!("Discord to IRC send error: {}", e)
                    }
                }
            }
        } else {
            debug!("DIS> {:?}", msg);
        }
    }
}
