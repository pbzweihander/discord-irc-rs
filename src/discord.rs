use {
    crate::config::*,
    async_std::task::spawn,
    futures::{channel::mpsc::UnboundedSender, prelude::*},
    serenity::{model::channel::Message, prelude::*},
    std::borrow::Cow,
};

pub struct DiscordHandler {
    config: DiscordConfig,
    irc_channel: String,
    irc_writer: UnboundedSender<String>,
}

impl DiscordHandler {
    pub fn new(
        config: DiscordConfig,
        irc_channel: String,
        irc_writer: UnboundedSender<String>,
    ) -> Self {
        DiscordHandler {
            config,
            irc_channel,
            irc_writer,
        }
    }
}

impl EventHandler for DiscordHandler {
    fn message(&self, ctx: Context, msg: Message) {
        if !msg.author.bot && msg.channel_id == self.config.channel_id {
            let mut content = msg.content;

            for user in msg.mentions {
                content = content.replace(&format!("{}", user.id), &user.name);
            }
            for role_id in msg.mention_roles {
                if let Some(role) = role_id.to_role_cached(&ctx.cache) {
                    content = content.replace(&format!("{}", role.id), &role.name);
                }
            }

            let lines = content
                .split('\n')
                .map(|s| Cow::Borrowed(s))
                .chain(msg.attachments.into_iter().map(|at| at.url).map(Into::into));

            for line in lines {
                let name = &msg.author.name;

                if self.config.ignores.contains(name) {
                    debug!("DIS| <{}(ignored)> {}", msg.author.name, line);
                } else {
                    info!("DIS> <{}> {}", msg.author.name, line);

                    let mut writer = self.irc_writer.clone();
                    let msg = format!(
                        "PRIVMSG {} :<{}> {}\n",
                        self.irc_channel, msg.author.name, line,
                    );
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
