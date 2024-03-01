//! Top level commands shared that are always available

use tracing::{info, instrument};

use crate::{
    commands::{tracing_handler_end, tracing_handler_start},
    Context,
};

/// Responds with "pong"
#[poise::command(slash_command, prefix_command, track_edits)]
#[instrument(name = "ping", skip(ctx))]
pub async fn ping(ctx: Context<'_>) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    let latency = ctx.ping().await;
    if latency.is_zero() {
        info!("latency not available yet");
        ctx.say("pong!").await?;
    } else {
        let msg = format!("pong! Bot gateway heartbeat latency is {latency:?}");
        info!(msg);
        ctx.say(msg).await?;
    }
    tracing_handler_end()
}

/// Responds with how long the bot has been running for
#[poise::command(hide_in_help, slash_command, prefix_command, track_edits)]
#[instrument(name = "uptime", skip(ctx))]
pub async fn uptime(ctx: Context<'_>) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    let msg = format!("{:?}", ctx.data().start_instant().elapsed());
    ctx.say(msg).await?;
    tracing_handler_end()
}

/// Show help menu
#[poise::command(prefix_command, track_edits, slash_command)]
#[instrument(name = "help", skip(ctx))]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    let config = Default::default();
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    tracing_handler_end()
}

/// Returns the version of the bot
#[poise::command(prefix_command, track_edits, slash_command)]
#[instrument(name = "version", skip(ctx))]
pub async fn version(ctx: Context<'_>) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    let msg = format!("Bot version is {}", version::version!());
    info!(msg);
    ctx.say(msg).await?;
    tracing_handler_end()
}
