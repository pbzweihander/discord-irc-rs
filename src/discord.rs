use {
    crate::{config::*, utils::normalize_irc_nickname},
    async_std::task::spawn,
    futures::{channel::mpsc::UnboundedSender, prelude::*},
    serenity::{model::channel::Message, prelude::*},
    std::borrow::Cow,
};

pub struct DiscordHandler {
    config: DiscordConfig,
    irc_config: IrcConfig,
    irc_writer: UnboundedSender<String>,
}

impl DiscordHandler {
    pub fn new(
        config: DiscordConfig,
        irc_config: IrcConfig,
        irc_writer: UnboundedSender<String>,
    ) -> Self {
        DiscordHandler {
            config,
            irc_config,
            irc_writer,
        }
    }
}

impl EventHandler for DiscordHandler {
    fn message(&self, ctx: Context, msg: Message) {
        if !msg.author.bot && msg.channel_id == self.config.channel_id {
            let Message {
                mut content,
                guild_id,
                author,
                attachments,
                mentions,
                mention_roles,
                ..
            } = msg;
            let Context { http, cache, .. } = ctx;
            let id = author.id.0;
            let name = guild_id
                .and_then(|guild_id| author.nick_in(http, guild_id))
                .unwrap_or(author.name);

            for user in mentions {
                content = content.replace(&format!("{}", user.id), &user.name);
            }
            for role_id in mention_roles {
                if let Some(role) = role_id.to_role_cached(&cache) {
                    content = content.replace(&format!("{}", role.id), &role.name);
                }
            }

            let lines = content
                .split('\n')
                .map(|s| Cow::Borrowed(s))
                .chain(attachments.into_iter().map(|at| at.url).map(Into::into));

            if self.config.ignores.contains(&name) {
                for line in lines {
                    debug!("DIS| <{}(ignored)> {}", name, line);
                }
            } else {
                let is_ozinger = self.irc_config.ozinger.is_some();
                let channel = self.irc_config.channel.clone();
                let lines: Vec<String> = lines
                    .map(move |line| {
                        info!("DIS> <{}> {}", name, line);
                        if is_ozinger {
                            format!(
                                "FAKEMSG {}＠d!{:x}@pbzweihander/discord-irc-rs {} :{}\n",
                                normalize_irc_nickname(&name),
                                id,
                                channel,
                                line,
                            )
                        } else {
                            format!("PRIVMSG {} :<{}> {}\n", channel, name, line)
                        }
                    })
                    .collect();

                let mut writer = self.irc_writer.clone();

                let fut = async move {
                    for line in lines {
                        writer
                            .send(line)
                            .unwrap_or_else(|err| error!("mpsc send error: {}", err))
                            .await;
                    }
                };

                spawn(fut);
            }
        } else {
            debug!("DIS> {:?}", msg);
        }
    }
}
