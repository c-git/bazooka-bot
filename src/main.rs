use std::sync::Arc;

use anyhow::Context as _;
use bazooka_bot::{commands_list, get_secret_discord_token, Data, SharedConfig, StartupConfig};
use poise::serenity_prelude::{ClientBuilder, GatewayIntents};
use secrecy::ExposeSecret;
use shuttle_persist::PersistInstance;
use shuttle_secrets::SecretStore;
use shuttle_serenity::ShuttleSerenity;
use tracing::{error, info, warn};
use version::version;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
    #[shuttle_persist::Persist] persist: PersistInstance,
) -> ShuttleSerenity {
    info!("Bot version is {}", version::version!());

    // Load setup values
    let discord_token = get_secret_discord_token(&secret_store)?;
    let startup_config =
        StartupConfig::try_new(&secret_store).context("failed to create setup config")?;
    let shared_config =
        SharedConfig::try_new(&secret_store, persist).context("failed to created shared_config")?;

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
                let connect_msg = format!("{} is connected! Version: {}", ready.user.name, version!());
                info!("{connect_msg}");
                if let Some(channel) = startup_config.channel_bot_status{
                    channel.say(ctx, connect_msg).await?;
                } else{
                    warn!("Not sending connection notification because channel_bot_status not set");
                }
                let data = Data::new(shared_config);
                info!("END OF SETUP CLOSURE");
                Ok(data)
            })
        })
        .build();

    let client = ClientBuilder::new(
        discord_token.expose_secret(),
        GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT,
    )
    .framework(framework)
    .await
    .map_err(shuttle_runtime::CustomError::new)?;

    Ok(client.into())
}
