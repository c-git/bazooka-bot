//! Top level commands shared that are always available

use tracing::instrument;

use crate::{commands::fn_start_tracing, Context};

/// Responds with "pong"
#[poise::command(slash_command, prefix_command, track_edits)]
#[instrument(name = "ping", skip(ctx))]
pub async fn ping(ctx: Context<'_>) -> anyhow::Result<()> {
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
