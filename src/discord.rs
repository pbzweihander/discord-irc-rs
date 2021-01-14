use {
    crate::{config::*, utils::normalize_irc_nickname},
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

#[serenity::async_trait]
impl EventHandler for DiscordHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        if !msg.author.bot && msg.channel_id == self.config.channel_id {
            let Context { http, cache, .. } = ctx;

            let content = msg.content_safe(&cache).await;
            let id = msg.author.id.0;
            let name = msg.author_nick(&http).await.unwrap_or(msg.author.name);

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

                for line in lines {
                    writer
                        .send(line)
                        .unwrap_or_else(|err| error!("mpsc send error: {}", err))
                        .await;
                }
            }
        } else {
            debug!("DIS> {:?}", msg);
        }
    }
}
