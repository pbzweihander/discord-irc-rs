#[macro_use]
extern crate log;

mod config;
mod discord;
mod irc;
mod webhook;

use {
    crate::{config::*, discord::*, irc::*},
    failure::{err_msg, Fallible},
    futures::{compat::*, prelude::*},
    std::{env::args, net::ToSocketAddrs, process::exit, thread},
    tokio::{
        net::TcpStream,
        prelude::{Future as _, Stream as _},
        sync::mpsc,
    },
    yaircc::*,
};

#[macro_export]
macro_rules! write_irc {
    ($writer:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        $writer.raw(msg).await?;
    }
}

async fn spawn_irc(
    config: IrcConfig,
    discord_webhook: String,
    irc_receiver: mpsc::UnboundedReceiver<String>,
) -> Fallible<()> {
    let tcp_stream = TcpStream::connect(&config.url.to_socket_addrs()?.next().unwrap())
        .compat()
        .await?;
    let irc_stream = IrcStream::new(Compat01As03::new(tcp_stream), encoding::all::UTF_8);
    let writer = irc_stream.writer();
    let writer_clone = writer.clone();

    write_irc!(
        writer,
        "USER {} 8 * :{}\n",
        config.username,
        config.realname
    );
    write_irc!(writer, "NICK {}\n", config.nickname);

    tokio::spawn(
        irc_stream
            .err_into()
            .and_then(move |msg| {
                handle_irc(msg, writer.clone(), discord_webhook.clone(), config.clone())
            })
            .try_collect()
            .unwrap_or_else(|err| error!("IrcStream error: {}", err))
            .map(|_| exit(1))
            .boxed()
            .compat(),
    );

    tokio::spawn(
        irc_receiver
            .map_err(Into::into)
            .for_each(move |msg| send_irc(writer_clone.clone(), msg).boxed().compat())
            .map_err(|err| error!("Irc send error: {}", err))
            .map(|_| exit(1)),
    );

    Ok(())
}

fn main() -> Fallible<()> {
    env_logger::try_init()?;

    let args: Vec<_> = args().take(2).collect();
    if args.len() != 2 {
        return Err(err_msg(format!("USAGE: {} <CONFIG_PATH>", args[0])));
    }
    let Config { irc, discord } = Config::from_path(&args[1])?;
    let (irc_sender, irc_receiver) = mpsc::unbounded_channel();

    let irc_fut =
        spawn_irc(irc.clone(), discord.webhook_url.clone(), irc_receiver).map(Result::unwrap);

    thread::spawn(move || {
        tokio::run(irc_fut.unit_error().boxed().compat());
        exit(1);
    });

    let mut client = serenity::Client::new(
        &discord.token.clone(),
        DiscordHandler::new(discord, irc.channel, irc_sender),
    )?;

    client.start()?;

    exit(1)
}
