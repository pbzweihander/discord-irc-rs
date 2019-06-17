#![recursion_limit = "128"]
#![feature(async_await, try_blocks)]

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

fn main() -> Fallible<()> {
    env_logger::try_init()?;

    let args: Vec<_> = args().take(2).collect();
    if args.len() != 2 {
        return Err(err_msg(format!("USAGE: {} <CONFIG_PATH>", args[0])));
    }
    let Config { irc, discord } = Config::from_path(&args[1])?;
    let irc_channel = irc.channel.clone();
    let discord_webhook = discord.webhook_url.clone();

    let (irc_sender, irc_receiver) = mpsc::unbounded_channel();

    let async_fut = async {
        let fallible: Fallible<()> = try {
            let tcp_stream = TcpStream::connect(&irc.url.to_socket_addrs()?.next().unwrap())
                .compat()
                .await?;
            let irc_stream = IrcStream::new(Compat01As03::new(tcp_stream), encoding::all::UTF_8);
            let writer = irc_stream.writer();
            let writer_clone = writer.clone();

            writer
                .raw(format!("USER {} 8 * :{}\n", irc.username, irc.realname))
                .await?;
            writer.raw(format!("NICK {}\n", irc.nickname)).await?;

            tokio::spawn(
                irc_stream
                    .err_into()
                    .and_then(move |msg| {
                        handle_irc(
                            irc.clone(),
                            msg,
                            writer_clone.clone(),
                            discord_webhook.clone(),
                        )
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
                    .for_each(move |msg| send_irc(writer.clone(), msg).boxed().compat())
                    .map_err(|err| error!("Irc send error: {}", err))
                    .map(|_| exit(1)),
            );
        };
        fallible.unwrap()
    };

    thread::spawn(move || {
        tokio::run(async_fut.unit_error().boxed().compat());
        exit(1);
    });

    let mut client = serenity::Client::new(
        &discord.token.clone(),
        DiscordHandler::new(discord, irc_channel, irc_sender),
    )?;

    client.start()?;

    exit(1)
}
