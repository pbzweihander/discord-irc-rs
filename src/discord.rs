use {
    crate::config::*,
    serenity::{model::channel::Message, prelude::*},
    std::borrow::Cow,
    tokio::{
        prelude::{Future, Sink},
        sync::mpsc::UnboundedSender,
    },
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
                info!("DIS> <{}> {}", msg.author.name, line);

                let _ = self
                    .irc_writer
                    .clone()
                    .send(format!(
                        "PRIVMSG {} :<{}> {}\n",
                        self.irc_channel, msg.author.name, line,
                    ))
                    .map(|_| ())
                    .map_err(|err| error!("mpsc send error: {}", err))
                    .wait();
            }
        } else {
            debug!("DIS> {:?}", msg);
        }
    }
}
