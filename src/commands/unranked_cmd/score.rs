//! Groups the commands related to the scoring functionality for unranked

use crate::{
    commands::{call_to_parent_command, tracing_handler_start},
    Context,
};
use tracing::instrument;

#[poise::command(prefix_command, slash_command, subcommands("set", "remove", "results"))]
#[instrument(name = "unranked-score", skip(ctx))]
/// Commands related to scoring during the event
pub async fn score(ctx: Context<'_>) -> anyhow::Result<()> {
    call_to_parent_command(ctx).await
}

#[poise::command(prefix_command, slash_command)]
#[instrument(name = "unranked-score-set", skip(ctx))]
/// Set or overwrite your score
pub async fn set(ctx: Context<'_>, score: u8) -> anyhow::Result<()> {
    tracing_handler_start(&ctx);
    todo!()
}

#[poise::command(prefix_command, slash_command)]
#[instrument(name = "unranked-score-remove", skip(ctx))]
/// Remove your score
pub async fn remove(ctx: Context<'_>) -> anyhow::Result<()> {
    tracing_handler_start(&ctx);
    todo!()
}

#[poise::command(prefix_command, slash_command)]
#[instrument(name = "unranked-score-results", skip(ctx))]
/// Show the current score results
pub async fn results(ctx: Context<'_>) -> anyhow::Result<()> {
    tracing_handler_start(&ctx);
    todo!()
}
