use std::sync::Arc;

use anyhow::Context as _;
use data::Data;
use poise::serenity_prelude::GuildId;
use poise::serenity_prelude::{ClientBuilder, GatewayIntents};
use shuttle_persist::PersistInstance;
use shuttle_secrets::SecretStore;
use shuttle_serenity::ShuttleSerenity;
use tracing::{info, instrument};

mod commands;
mod data;

use commands::{unranked, Unranked};

type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

/// Common info added to tracing for functions
fn fn_start_tracing(ctx: &Context) {
    info!("Author: {}", ctx.author().name);
}

/// Standardized response to a call to a parent function (not callable by slash command)
async fn call_to_parent_command(ctx: Context<'_>) -> anyhow::Result<()> {
    info!("{} called a parent command", ctx.author().name);
    ctx.say("requires subcommand see /help").await?;
    Ok(())
}

/// Responds with "pong"
#[poise::command(slash_command, prefix_command, track_edits)]
#[instrument(name = "ping", skip(ctx))]
async fn ping(ctx: Context<'_>) -> anyhow::Result<()> {
    fn_start_tracing(&ctx);
    ctx.say("pong!").await?;
    Ok(())
}

/// Show help menu
#[poise::command(prefix_command, track_edits, slash_command)]
#[instrument(name = "help", skip(ctx))]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> anyhow::Result<()> {
    fn_start_tracing(&ctx);
    let config = Default::default();
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
    #[shuttle_persist::Persist] persist: PersistInstance,
) -> ShuttleSerenity {
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
            commands: vec![ping(), help(), unranked()],
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
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    GuildId::new(guild_id),
                )
                .await?;
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
