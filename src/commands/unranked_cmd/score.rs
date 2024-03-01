//! Groups the commands related to the scoring functionality for unranked

use crate::{
    commands::{is_auth, tracing_handler_end, tracing_handler_start},
    model::{unranked::scores::ScoreValue, user_serde::UserRecordSupport as _},
    Context,
};
use std::fmt::Debug;
use tracing::{info, instrument};

#[poise::command(
    prefix_command,
    slash_command,
    track_edits,
    subcommands("set", "remove", "leader_board", "message")
)]
#[instrument(name = "unranked-score", skip(ctx))]
/// Commands related to scoring during the event and if called using `bbur score` sets the score
pub async fn score(ctx: Context<'_>, value: ScoreValue) -> anyhow::Result<()> {
    do_set_score(ctx, value).await
}

#[poise::command(prefix_command, slash_command)]
#[instrument(name = "unranked-score-remove", skip(ctx))]
/// Remove your score
pub async fn remove(ctx: Context<'_>) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    let did_remove = ctx
        .data()
        .unranked
        .score_remove(&ctx.author_to_user_record().await)?;
    display_scores_with_msg(
        &ctx,
        if did_remove {
            "Score Removed"
        } else {
            "Score not Found"
        },
    )
    .await?;
    tracing_handler_end()
}

#[poise::command(prefix_command, slash_command, aliases("disp"))]
#[instrument(name = "unranked-score-leader_board", skip(ctx))]
/// Show the current leader_board
pub async fn leader_board(ctx: Context<'_>) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    display_scores(&ctx).await?;
    tracing_handler_end()
}

#[poise::command(prefix_command, slash_command, track_edits)]
#[instrument(name = "unranked-score-set", skip(ctx))]
/// Set or overwrite your score
pub async fn set(ctx: Context<'_>, score: ScoreValue) -> anyhow::Result<()> {
    do_set_score(ctx, score).await
}

#[poise::command(prefix_command, slash_command, track_edits, check = "is_auth")]
#[instrument(name = "unranked-score-message", skip(ctx))]
/// Set message displayed with scores (Replaces current message)
pub async fn message(ctx: Context<'_>, #[rest] msg: String) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    ctx.data()
        .unranked
        .score_message(ctx.author_id_number(), msg)?;
    display_scores_with_msg(&ctx, "Message Set").await?;
    tracing_handler_end()
}

async fn do_set_score(ctx: Context<'_>, score: ScoreValue) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    ctx.data()
        .unranked
        .score_set(ctx.author_to_user_record().await, score)?;
    display_scores_with_msg(&ctx, "Score Set").await?;
    tracing_handler_end()
}

#[instrument(skip(ctx))]
async fn display_scores_with_msg<S: Into<String> + Debug>(
    ctx: &Context<'_>,
    msg: S,
) -> anyhow::Result<()> {
    info!("START");
    display_scores(ctx).await?;
    ctx.say(msg).await?;
    tracing_handler_end()
}

async fn display_scores(ctx: &Context<'_>) -> anyhow::Result<()> {
    ctx.say(ctx.data().unranked.scores_as_string()?).await?;
    Ok(())
}
