//! Groups the commands related to the unranked challenge

use tracing::{info, instrument};

use self::{idea::idea, score::score};
use crate::{
    commands::{call_to_parent_command, is_auth, tracing_handler_end, tracing_handler_start},
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
    subcommands("idea", "score", "start_event", "schedule_start_event")
)]
#[instrument(name = "unranked", skip(ctx))]
/// Commands related to the Unranked Challenge [aliases("ur")]
pub async fn unranked(ctx: Context<'_>) -> anyhow::Result<()> {
    call_to_parent_command(ctx).await
}

#[poise::command(
    hide_in_help,
    prefix_command,
    slash_command,
    track_edits,
    check = "is_auth"
)]
#[instrument(name = "unranked-start_event", skip(ctx))]
pub async fn start_event(ctx: Context<'_>) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    // do_start_event()?;
    tracing_handler_end()
}

#[instrument]
fn do_start_event() -> anyhow::Result<()> {
    info!("START");
    todo!()
}

#[poise::command(
    hide_in_help,
    prefix_command,
    slash_command,
    track_edits,
    check = "is_auth"
)]
#[instrument(name = "unranked-schedule_start_event", skip(ctx))]
pub async fn schedule_start_event(ctx: Context<'_>) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    todo!()
}
