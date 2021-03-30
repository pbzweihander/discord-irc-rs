#[macro_use]
extern crate tracing;

mod config;
mod discord;
mod irc;
mod utils;

use std::env::args;
use std::process::exit;
use std::sync::Arc;

use anyhow::{bail, Result};
use futures::prelude::*;
use libirc::client::Client;

async fn irc_handler_future(
    mut irc_client: Client,
    discord_http: Arc<serenity::http::client::Http>,
    discord_webhook_id: u64,
    discord_webhook_token: String,
    irc_config: config::IrcConfig,
) -> Result<()> {
    irc_client
        .stream()?
        .err_into()
        .and_then(|msg| {
            let discord_http = discord_http.clone();
            let discord_webhook_token = discord_webhook_token.clone();
            let irc_config = irc_config.clone();
            async move {
                irc::handle_irc(
                    msg,
                    &discord_http,
                    discord_webhook_id,
                    discord_webhook_token,
                    irc_config,
                )
                .await
            }
        })
        .map(|res| {
            if let Err(err) = res {
                error!("IrcStream error: {}", err);
            }
        })
        .collect::<()>()
        .await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args: Vec<_> = args().take(2).collect();
    if args.len() != 2 {
        bail!("USAGE: {} <CONFIG_PATH>", args[0]);
    }
    let config::Config {
        irc: irc_config,
        discord: discord_config,
    } = config::Config::from_path(&args[1])?;

    let irc_client = Client::from_config(irc_config.connection.clone()).await?;
    let irc_sender = irc_client.sender();
    irc_client.identify()?;
    irc_client.send_join(&irc_config.channel)?;

    let mut discord_client = serenity::Client::builder(&discord_config.token.clone())
        .event_handler(discord::DiscordHandler::new(
            discord_config.clone(),
            irc_config.clone(),
            irc_sender,
        ))
        .await?;

    let irc_fut = irc_handler_future(
        irc_client,
        discord_client.cache_and_http.http.clone(),
        discord_config.webhook_id,
        discord_config.webhook_token,
        irc_config,
    );

    let discord_fut = discord_client.start().map_err(anyhow::Error::from);

    futures::future::try_join(irc_fut, discord_fut).await?;

    exit(1)
}
