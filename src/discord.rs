use {
    crate::config::*,
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

            for line in lines {
                if self.config.ignores.contains(&name) {
                    debug!("DIS| <{}(ignored)> {}", name, line);
                } else {
                    info!("DIS> <{}> {}", name, line);

                    let mut writer = self.irc_writer.clone();
                    let msg = match self.irc_config.ozinger {
                        Some(_) => format!("FAKEMSG {}＠d!{:x}@pbzweihander/discord-irc-rs {} :{}\n",
                            name.replace("!", "ǃ").replace("@", "＠"), // U+0021 -> U+01C3, U+0040 -> U+FE6B
                            author.id.0,
                            self.irc_config.channel,
                            line,
                        ),
                        None => format!("PRIVMSG {} :<{}> {}\n", self.irc_config.channel, name, line, ),
                    };
                    spawn(async move {
                        writer
                            .send(msg)
                            .unwrap_or_else(|err| error!("mpsc send error: {}", err))
                            .await
                    });
                }
            }
        } else {
            debug!("DIS> {:?}", msg);
        }
    }
}
