#[macro_use]
extern crate tracing;

mod config;
mod discord;
mod format;
mod irc;
mod utils;

use std::env::args;
use std::process::exit;
use std::sync::Arc;

use anyhow::{bail, Result};
use futures::prelude::*;
use libirc::client::Client;
use serenity::client::bridge::gateway::GatewayIntents;

async fn irc_handler_future(
    mut irc_client: Client,
    discord_http: Arc<serenity::CacheAndHttp>,
    irc_config: config::IrcConfig,
    discord_config: config::DiscordConfig,
) -> Result<()> {
    let irc_sender = irc_client.sender();
    irc_client
        .stream()?
        .err_into()
        .and_then(|msg| {
            let irc_sender = irc_sender.clone();
            let discord_http = discord_http.clone();
            let irc_config = irc_config.clone();
            let discord_config = discord_config.clone();
            async move {
                irc::handle_irc(msg, irc_sender, &discord_http, irc_config, discord_config).await
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

    let mut intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES;
    if irc_config.auto_detect_avatar {
        intents |= GatewayIntents::GUILD_MEMBERS | GatewayIntents::GUILD_PRESENCES;
    }

    let mut discord_client = serenity::Client::builder(&discord_config.token.clone())
        .event_handler(discord::DiscordHandler::new(
            discord_config.clone(),
            irc_config.clone(),
            irc_sender,
        ))
        .intents(intents)
        .await?;

    let irc_fut = irc_handler_future(
        irc_client,
        discord_client.cache_and_http.clone(),
        irc_config,
        discord_config,
    );

    let discord_fut = discord_client.start().map_err(anyhow::Error::from);

    futures::future::try_join(irc_fut, discord_fut).await?;

    exit(1)
}
