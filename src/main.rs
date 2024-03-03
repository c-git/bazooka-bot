use std::sync::Arc;

use anyhow::Context as _;
use bazooka_bot::{commands_list, AccessSecrets as _, Data, SharedConfig, StartupConfig};
use poise::serenity_prelude::{ClientBuilder, GatewayIntents};
use shuttle_persist::PersistInstance;
use shuttle_secrets::SecretStore;
use shuttle_serenity::ShuttleSerenity;
use tracing::info;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
    #[shuttle_persist::Persist] persist: PersistInstance,
) -> ShuttleSerenity {
    info!("Bot version is {}", version::version!());

    // Load setup values
    let discord_token = secret_store.access_secret_string("DISCORD_TOKEN")?;
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
                } else {
                    info!("Development run detected going to register guild: {guild_id}");
                    poise::builtins::register_in_guild(
                        ctx,
                        &framework.options().commands,
                        startup_config.guild_id,
                    )
                    .await
                    .with_context(|| {
                        format!(
                            "failed to register {:?} in guild: {guild_id}",
                            ready.user.name
                        )
                    })?;
                }
                info!("{} is connected!", ready.user.name);
                Ok(Data::new(shared_config))
            })
        })
        .build();

    let client = ClientBuilder::new(
        discord_token,
        GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT,
    )
    .framework(framework)
    .await
    .map_err(shuttle_runtime::CustomError::new)?;

    Ok(client.into())
}
