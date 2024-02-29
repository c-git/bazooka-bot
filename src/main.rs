use std::sync::Arc;

use anyhow::Context as _;
use bazooka_bot::{commands_list, Data};
use poise::serenity_prelude::GuildId;
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
    // Get the discord token and guild_id set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;
    let guild_id: u64 = secret_store
        .get("GUILD_ID")
        .context("'GUILD_ID' was not found")?
        .parse()
        .context("failed to parse GUILD_ID")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands_list(),
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
                info!("Going to register guild: {guild_id}");
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    GuildId::new(guild_id),
                )
                .await
                .with_context(|| {
                    format!(
                        "failed to register {:?} in guild: {guild_id}",
                        ready.user.name
                    )
                })?;
                info!("{} is connected!", ready.user.name);
                Ok(Data::new(persist))
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
