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
    fn message(&self, _: Context, msg: Message) {
        if !msg.author.bot && msg.channel_id == self.config.channel_id {
            let msg_iter = msg
                .content
                .split('\n')
                .map(|s| Cow::Borrowed(s))
                .chain(msg.attachments.into_iter().map(|at| at.url).map(Into::into));

            for line in msg_iter {
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
