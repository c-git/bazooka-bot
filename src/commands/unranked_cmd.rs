//! Groups the commands related to the unranked challenge

use tracing::instrument;

use self::score::score;
use crate::{
    commands::{call_to_parent_command, fn_start_tracing},
    Context,
};
use idea::idea;

mod idea;
mod score;

#[poise::command(
    prefix_command,
    slash_command,
    track_edits,
    subcommands("idea", "score", "schedule_reset")
)]
#[instrument(name = "unranked", skip(ctx))]
/// Commands related to the Unranked Challenge
pub async fn unranked(ctx: Context<'_>) -> anyhow::Result<()> {
    call_to_parent_command(ctx).await
}

#[poise::command(prefix_command, slash_command, track_edits)]
#[instrument(name = "schedule_reset", skip(ctx))]
pub async fn schedule_reset(ctx: Context<'_>) -> anyhow::Result<()> {
    // TODO 2: Require Auth
    fn_start_tracing(&ctx);
    todo!()
}
