use std::sync::Arc;

use anyhow::Context as _;
use data::Data;
use poise::serenity_prelude::{self as serenity, GuildId};
use poise::serenity_prelude::{ClientBuilder, GatewayIntents};
use shuttle_persist::PersistInstance;
use shuttle_secrets::SecretStore;
use shuttle_serenity::ShuttleSerenity;
use tracing::info;

mod data;

type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

/// Responds with "pong"
#[poise::command(slash_command, prefix_command, track_edits)]
async fn ping(ctx: Context<'_>) -> anyhow::Result<()> {
    ctx.say("pong!").await?;
    Ok(())
}

/// Responds with debug info
#[poise::command(hide_in_help, slash_command, prefix_command, track_edits)]
async fn debug(ctx: Context<'_>) -> anyhow::Result<()> {
    let response = format!("Author: `{}`\nPrefix:{}", ctx.author().name, ctx.prefix(),);
    info!(response);
    ctx.say(response).await?;
    info!("ctx: {:?}", ctx);
    Ok(())
}

/// Responds with "world!"
#[poise::command(slash_command, prefix_command, track_edits)]
async fn hello(ctx: Context<'_>) -> anyhow::Result<()> {
    info!("{} says hi", ctx.author().name);
    ctx.say("world!").await?;
    Ok(())
}

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command, track_edits)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> anyhow::Result<()> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    info!(response);
    ctx.say(response).await?;
    Ok(())
}

/// Saves a message
#[poise::command(slash_command, prefix_command, track_edits)]
async fn save_msg(ctx: Context<'_>, message: String) -> anyhow::Result<()> {
    ctx.data().set_message(message)?;
    ctx.say("Message Saved").await?;
    Ok(())
}

/// Returns the saved message
#[poise::command(slash_command, prefix_command, track_edits)]
async fn load_msg(ctx: Context<'_>) -> anyhow::Result<()> {
    ctx.say(ctx.data().message()?).await?;
    Ok(())
}

/// Show help menu
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> anyhow::Result<()> {
    let config = Default::default();
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
    #[shuttle_persist::Persist] persist: PersistInstance,
) -> ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                hello(),
                age(),
                ping(),
                debug(),
                save_msg(),
                load_msg(),
                help(),
            ],
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
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                // poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    GuildId::new(839130241040515072),
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
