//! Groups the commands related to the unranked challenge

use tracing::instrument;

use self::{idea::idea, score::score};
use crate::{
    commands::{call_to_parent_command, tracing_handler_start},
    Context,
};

mod idea;
mod score;

#[poise::command(
    prefix_command,
    slash_command,
    track_edits,
    aliases("ur"),
    subcommand_required,
    subcommands(
        "idea", 
        "score", 
        // TODO 2: Enable functions
        // "schedule_reset"
    )
)]
#[instrument(name = "unranked", skip(ctx))]
/// Commands related to the Unranked Challenge
pub async fn unranked(ctx: Context<'_>) -> anyhow::Result<()> {
    call_to_parent_command(ctx).await
}

#[poise::command(hide_in_help, prefix_command, slash_command, track_edits)]
#[instrument(name = "schedule_reset", skip(ctx))]
pub async fn schedule_reset(ctx: Context<'_>) -> anyhow::Result<()> {
    // TODO 2: Require Auth
    tracing_handler_start(&ctx).await;
    todo!()
}
