//! Groups the commands related to the scoring functionality for unranked

use crate::{
    commands::{is_auth, tracing_handler_end, tracing_handler_start},
    model::{
        unranked::scores::{ScoreValue, Scores},
        user_serde::UserRecordSupport as _,
    },
    Context,
};
use poise::{serenity_prelude::CreateEmbed, CreateReply};
use std::fmt::Debug;
use tracing::{info, instrument};

#[poise::command(
    prefix_command,
    slash_command,
    track_edits,
    subcommands("set", "remove", "leader_board", "message", "reset")
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
            "Score removed"
        } else {
            "**Score not found**"
        },
    )
    .await?;
    tracing_handler_end()
}

#[poise::command(prefix_command, slash_command, aliases("disp"))]
#[instrument(name = "unranked-score-leader_board", skip(ctx))]
/// Show the current leader_board [aliases("disp")]
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

#[poise::command(
    prefix_command,
    slash_command,
    track_edits,
    aliases("msg"),
    guild_only = true,
    check = "is_auth"
)]
#[instrument(name = "unranked-score-message", skip(ctx))]
/// Set message displayed with scores (Replaces current message) [aliases("msg")]
pub async fn message(ctx: Context<'_>, #[rest] msg: Option<String>) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    let is_cleared = msg.is_none();
    let msg = msg.unwrap_or_default();
    ctx.data()
        .unranked
        .scores_message(ctx.author_id_number(), msg)?;
    display_scores_with_msg(
        &ctx,
        if is_cleared {
            "Message cleared"
        } else {
            "Message set"
        },
    )
    .await?;
    tracing_handler_end()
}

#[poise::command(prefix_command, guild_only = true, check = "is_auth")]
#[instrument(name = "unranked-score-reset", skip(ctx))]
/// Sets scores back to the default
pub async fn reset(ctx: Context<'_>) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    do_scores_reset(&ctx).await?;
    tracing_handler_end()
}

#[instrument(skip(ctx))]
pub async fn do_scores_reset(ctx: &Context<'_>) -> anyhow::Result<()> {
    info!("START");
    display_scores_with_msg(ctx, "Scores before reset").await?;
    ctx.data().unranked.scores_reset()?;
    display_scores_with_msg(ctx, "---\nScores reset").await?;
    tracing_handler_end()
}

async fn do_set_score(ctx: Context<'_>, score: ScoreValue) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    ctx.data()
        .unranked
        .score_set(ctx.author_to_user_record().await, score)?;
    display_scores_with_msg(&ctx, "Score set").await?;
    tracing_handler_end()
}

#[instrument(skip(ctx))]
async fn do_display_scores<S: Into<String> + Debug>(
    ctx: &Context<'_>,
    extra_msg: Option<S>,
) -> anyhow::Result<()> {
    info!("START");
    let mut builder = display_generate(ctx)?;
    if let Some(msg) = extra_msg {
        builder = builder.content(msg);
    }
    ctx.send(builder).await?;
    tracing_handler_end()
}

#[instrument(skip(ctx))]
async fn display_scores_with_msg<S: Into<String> + Debug>(
    ctx: &Context<'_>,
    extra_msg: S,
) -> anyhow::Result<()> {
    do_display_scores(ctx, Some(extra_msg)).await
}

async fn display_scores(ctx: &Context<'_>) -> anyhow::Result<()> {
    do_display_scores::<&str>(ctx, None).await
}

fn display_generate(ctx: &Context<'_>) -> anyhow::Result<CreateReply> {
    let scores_as_string = ctx.data().unranked.scores_as_string()?;
    let embed = CreateEmbed::new()
        .title(Scores::DISPLAY_TITLE)
        .description(scores_as_string);
    Ok(CreateReply::default().embed(embed))
}
