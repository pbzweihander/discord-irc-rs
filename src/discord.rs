use {
    crate::config::*,
    serenity::{model::channel::Message, prelude::*},
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
            info!("DIS> <{}> {}", msg.author.name, msg.content);

            let _ = self
                .irc_writer
                .clone()
                .send(format!(
                    "PRIVMSG {} :<{}> {}\n",
                    self.irc_channel, msg.author.name, msg.content
                ))
                .map(|_| ())
                .map_err(|err| error!("mpsc send error: {}", err))
                .wait();
        } else {
            debug!("DIS> {:?}", msg);
        }
    }
}
