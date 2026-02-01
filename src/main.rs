use anyhow::Context as _;
use bazooka_bot::{ClapConfig, Data, SharedConfig, StartupConfig, commands_list, heartbeat};
use poise::serenity_prelude::{ClientBuilder, GatewayIntents};
use secrecy::ExposeSecret;
use shuttle_serenity::ShuttleSerenity;
use std::sync::Arc;
use tracing::{error, info, warn};
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, format::FmtSpan},
    prelude::*,
};
use version::version;

use clap::Parser;

#[shuttle_runtime::main]
async fn main() -> ShuttleSerenity {
    tracing_subscriber::registry()
        .with(fmt::layer().with_span_events(FmtSpan::NEW | FmtSpan::CLOSE))
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("zbus=warn,serenity=warn,info")),
        )
        .init();

    info!("Bot version is {}", version::version!());

    // Load setup values
    info!("Loading environment variables");
    loadenv::load().expect("failed to load .env file");
    let clap_config = ClapConfig::parse();
    info!("ClapConfig: {:?}", clap_config);

    let startup_config =
        StartupConfig::try_new(&clap_config).context("failed to create setup config")?;
    let shared_config =
        SharedConfig::try_new(&clap_config).context("failed to created shared_config")?;
    let discord_token = clap_config.discord_token;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands_list(),
            owners: startup_config.owners,
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("bb".into()),
                edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                    std::time::Duration::from_secs(3600),
                ))),
                case_insensitive_commands: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                if startup_config.is_production {
                    info!("Production run detected going to register globally");
                    poise::builtins::register_globally(ctx, &framework.options().commands)
                        .await
                        .context("failed to register the bot globally")?;
                } else if let Some(guild_id) = startup_config.registration_guild_id {
                    info!("Development run detected going to register guild: {guild_id}");
                    poise::builtins::register_in_guild(
                        ctx,
                        &framework.options().commands,
                        guild_id,
                    )
                    .await
                    .with_context(|| {
                        format!(
                            "failed to register {:?} in guild: {guild_id}",
                            ready.user.name
                        )
                    })?;
                } else {
                    error!("Development run detected but no guild ID found so slash commands NOT registered");
                }
                let connect_msg = format!(
                    "{} is connected! Version: {}\n{}", 
                    ready.user.name, version!(),
                    heartbeat::last_heartbeat_info().await,
                );
                info!("{connect_msg}");
                if let Some(channel) = shared_config.channel_bot_status{
                    channel.say(ctx, connect_msg).await?;
                } else{
                    warn!("Not sending connection notification because channel_bot_status not set");
                }
                let data = Data::new(shared_config, ctx.clone()).await;
                heartbeat::start_heartbeat();
                info!("END OF SETUP CLOSURE");
                Ok(data)
            })
        })
        .build();

    let client = ClientBuilder::new(
        discord_token.expose_secret(),
        // TODO 5: Try reducing intents
        GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT,
    )
    .framework(framework)
    .await
    .map_err(shuttle_runtime::CustomError::new)?;

    Ok(client.into())
}
