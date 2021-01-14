#[macro_use]
extern crate log;

mod config;
mod discord;
mod irc;
mod utils;

use {
    crate::{config::*, discord::*, irc::*},
    async_std::net::TcpStream,
    failure::{err_msg, Fallible},
    futures::{channel::mpsc, future::join, prelude::*},
    std::{env::args, net::ToSocketAddrs, process::exit},
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
    discord_http: std::sync::Arc<serenity::http::client::Http>,
    discord_webhook_id: u64,
    discord_webhook_token: String,
    irc_receiver: mpsc::UnboundedReceiver<String>,
) -> Fallible<()> {
    let tcp_stream = TcpStream::connect(&config.url.to_socket_addrs()?.next().unwrap()).await?;
    let irc_stream = IrcStream::new(tcp_stream, encoding::all::UTF_8);
    let writer = irc_stream.writer();
    let writer_clone = writer.clone();

    write_irc!(
        writer,
        "USER {} 8 * :{}\n",
        config.username,
        config.realname
    );
    if let Some(password) = &config.password {
        write_irc!(writer, "PASS {}\n", password);
    }
    write_irc!(writer, "NICK {}\n", config.nickname);

    let reader_fut = irc_stream
        .err_into()
        .and_then(move |msg| {
            handle_irc(
                discord_http.clone(),
                msg,
                writer.clone(),
                discord_webhook_id,
                discord_webhook_token.clone(),
                config.clone(),
            )
        })
        .try_collect()
        .unwrap_or_else(|err| error!("IrcStream error: {}", err));

    let writer_fut = irc_receiver
        .for_each(move |msg| {
            send_irc(writer_clone.clone(), msg)
                .unwrap_or_else(|err| error!("Irc send error: {}", err))
        });

    join(reader_fut, writer_fut).await;

    Ok(())
}

#[async_std::main]
async fn main() -> Fallible<()> {
    env_logger::try_init()?;

    let args: Vec<_> = args().take(2).collect();
    if args.len() != 2 {
        return Err(err_msg(format!("USAGE: {} <CONFIG_PATH>", args[0])));
    }
    let Config { irc, discord } = Config::from_path(&args[1])?;
    let (irc_sender, irc_receiver) = mpsc::unbounded();

    let webhook_id = discord.webhook_id;
    let webhook_token = discord.webhook_token.clone();
    let mut client = serenity::Client::builder(&discord.token.clone())
        .event_handler(DiscordHandler::new(discord, irc.clone(), irc_sender))
        .await?;

    let irc_fut = spawn_irc(
        irc,
        client.cache_and_http.http.clone(),
        webhook_id,
        webhook_token,
        irc_receiver,
    );
    let client_fut = client.start().map_err(failure::Error::from);

    futures::future::try_join(irc_fut, client_fut).await?;

    exit(1)
}
