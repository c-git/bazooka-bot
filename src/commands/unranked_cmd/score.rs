//! Groups the commands related to the scoring functionality for unranked

use crate::{
    commands::{tracing_handler_end, tracing_handler_start},
    model::unranked::ScoreValue,
    Context,
};
use std::fmt::Debug;
use tracing::{info, instrument};

#[poise::command(prefix_command, slash_command, subcommands("set", "remove", "results"))]
#[instrument(name = "unranked-score", skip(ctx))]
/// Commands related to scoring during the event and if called using `bbur score` sets the score
pub async fn score(ctx: Context<'_>, value: ScoreValue) -> anyhow::Result<()> {
    set_score(ctx, value).await
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

#[poise::command(prefix_command, slash_command)]
#[instrument(name = "unranked-score-set", skip(ctx))]
/// Set or overwrite your score
pub async fn set(ctx: Context<'_>, score: ScoreValue) -> anyhow::Result<()> {
    set_score(ctx, score).await
}

async fn set_score(ctx: Context<'_>, score: ScoreValue) -> anyhow::Result<()> {
    tracing_handler_start(&ctx);
    ctx.data()
        .unranked_score_set(&ctx, ctx.author().clone(), score)
        .await?;
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
    ctx.say(ctx.data().unranked_scores_as_string(ctx).await?)
        .await;
    Ok(())
}
