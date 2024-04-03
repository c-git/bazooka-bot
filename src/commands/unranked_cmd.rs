//! Groups the commands related to the unranked challenge

use poise::serenity_prelude::{CacheHttp, ChannelId};
use tracing::{info, instrument};

use self::{idea::idea, score::score};
use crate::{
    commands::{
        call_to_parent_command, is_auth, tracing_handler_end, tracing_handler_start,
        unranked_cmd::{
            idea::{display_ideas_channel, do_ideas_reset},
            score::{display_scores_channel, do_scores_reset},
        },
    },
    Context, Data,
};

mod idea;
mod score;

#[poise::command(
    prefix_command,
    slash_command,
    track_edits,
    aliases("ur"),
    subcommand_required,
    subcommands("idea", "score", "start_event")
)]
#[instrument(name = "unranked", skip(ctx))]
/// Commands related to the Unranked Challenge [aliases("ur")]
pub async fn unranked(ctx: Context<'_>) -> anyhow::Result<()> {
    call_to_parent_command(ctx).await
}

#[poise::command(hide_in_help, prefix_command, guild_only = true, check = "is_auth")]
#[instrument(name = "unranked-start_event", skip(ctx))]
/// Resets ideas and scores for the start of the new event and sets the message with the leading idea
pub async fn start_event(ctx: Context<'_>) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    ctx.reply("Request started").await?;
    do_start_event(ctx, ctx.channel_id(), ctx.data()).await?;
    tracing_handler_end()
}

#[instrument(skip(cache_http, data))]
pub async fn do_start_event(
    cache_http: impl CacheHttp,
    channel_id: ChannelId,
    data: &Data,
) -> anyhow::Result<()> {
    info!("START");
    channel_id
        .say(
            &cache_http,
            "Setting up for the start of a new unranked event",
        )
        .await?;

    display_ideas_channel(&cache_http, channel_id, data, true).await?;

    // Get the leading idea (winning at this point as it's the end) and the ones above the threshold
    let leading = data.inner.unranked.ideas_pop_leading()?;
    channel_id
        .say(&cache_http, "Extracting leading idea")
        .await?;

    // Do resets
    do_ideas_reset(&cache_http, channel_id, data).await?;
    do_scores_reset(&cache_http, channel_id, data).await?;

    // Get message for new scores
    let msg = if let Some(idea) = leading {
        idea.description
    } else {
        "Seems there were no ideas".to_string()
    };

    // Set message for new scores
    data.inner
        .unranked
        .scores_message(Default::default(), msg.clone())?;

    display_scores_channel(&cache_http, channel_id, data).await?;

    channel_id
        .say(
            &cache_http,
            format!("@here This season we will be doing:\n---\n> {msg}\n---"),
        )
        .await?;

    channel_id
        .say(&cache_http, "@here Setup successfully completed GLHF")
        .await?;

    tracing_handler_end()
}
