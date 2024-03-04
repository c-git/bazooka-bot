//! The commands related to the idea functionality for unranked

use std::{fmt::Debug, num::NonZeroUsize};

use poise::{
    serenity_prelude::{CacheHttp, ChannelId, CreateEmbed, CreateMessage},
    CreateReply,
};
use tracing::{info, instrument};

use crate::{
    commands::{
        call_to_parent_command, is_auth, tracing_handler_end, tracing_handler_start, Context,
    },
    model::{
        unranked::ideas::{IdeaId, Ideas},
        user_serde::UserRecordSupport as _,
    },
    sanitize_markdown, Data,
};

#[poise::command(
    prefix_command,
    slash_command,
    track_edits,
    subcommand_required,
    subcommands(
        "add",
        "edit",
        "remove",
        "vote",
        "unvote",
        "vote_all",
        "unvote_all",
        "display",
        "threshold",
        "reset",
    )
)]
#[instrument(name = "idea", skip(ctx))]
/// Commands related to ideas for the next unranked event
pub async fn idea(ctx: Context<'_>) -> anyhow::Result<()> {
    call_to_parent_command(ctx).await
}

#[poise::command(prefix_command, slash_command)]
#[instrument(name = "unranked-idea-add", skip(ctx))]
/// Adds a new idea
pub async fn add(ctx: Context<'_>, #[rest] description: String) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    let description = sanitize_markdown(description);
    ctx.data()
        .inner
        .unranked
        .idea_add(ctx.author_id_number(), description)?;
    display_ideas_with_msg(&ctx, "Idea added").await?;
    tracing_handler_end()
}

#[poise::command(prefix_command, slash_command, track_edits)]
#[instrument(name = "unranked-idea-edit", skip(ctx))]
/// Edits an idea you previously created
pub async fn edit(
    ctx: Context<'_>,
    id: NonZeroUsize,
    #[rest] new_description: String,
) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    let new_description = sanitize_markdown(new_description);
    let id: IdeaId = id.into();
    ctx.data()
        .inner
        .unranked
        .idea_edit(id, ctx.author_id_number(), new_description)?;
    display_ideas_with_msg(&ctx, "Idea Updated").await?;
    tracing_handler_end()
}

#[poise::command(prefix_command, slash_command)]
#[instrument(name = "unranked-idea-remove", skip(ctx))]
/// Removes and idea you previously created
pub async fn remove(ctx: Context<'_>, id: NonZeroUsize) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    let id: IdeaId = id.into();
    let old_idea = ctx
        .data()
        .inner
        .unranked
        .idea_remove(id, ctx.author_id_number(), false)?;
    display_ideas_with_msg(
        &ctx,
        format!(
            "Idea removed. It was: # {id} - {:?}",
            old_idea.description()
        ),
    )
    .await?;
    tracing_handler_end()
}

#[poise::command(prefix_command, slash_command)]
#[instrument(name = "unranked-idea-vote", skip(ctx))]
/// Adds your vote for the given idea (If you are currently voting for it nothing happens)
pub async fn vote(ctx: Context<'_>, id: NonZeroUsize) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    change_vote(ctx, id.into(), true).await
}

#[poise::command(prefix_command, slash_command)]
#[instrument(name = "unranked-idea-unvote", skip(ctx))]
/// Removes your vote for the given idea (If you are not currently voting for it nothing happens)
pub async fn unvote(ctx: Context<'_>, id: NonZeroUsize) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    change_vote(ctx, id.into(), false).await
}

#[poise::command(prefix_command, slash_command)]
#[instrument(name = "unranked-idea-vote_all", skip(ctx))]
/// Adds your vote for all current ideas
pub async fn vote_all(ctx: Context<'_>) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    change_vote_all(ctx, true).await
}

#[poise::command(prefix_command, slash_command)]
#[instrument(name = "unranked-idea-unvote_all", skip(ctx))]
/// Removes your vote for all ideas
pub async fn unvote_all(ctx: Context<'_>) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    change_vote_all(ctx, false).await
}

#[poise::command(prefix_command, slash_command, track_edits, aliases("disp"))]
#[instrument(name = "unranked-idea-unvote_all", skip(ctx))]
/// Displays all ideas optionally verbosely [aliases("disp")]
pub async fn display(ctx: Context<'_>, #[flag] is_verbose: bool) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    display_ideas(&ctx, is_verbose).await
}

#[poise::command(
    prefix_command,
    slash_command,
    track_edits,
    guild_only = true,
    check = "is_auth"
)]
#[instrument(name = "unranked-score-threshold", skip(ctx))]
/// Sets the Discard Threshold for resetting ideas.
/// Ideas with this many votes or less will be removed
pub async fn threshold(ctx: Context<'_>, threshold: usize) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    ctx.data().inner.unranked.idea_set_threshold(threshold)?;
    display_ideas_with_msg(&ctx, format!("Threshold set to {threshold}")).await?;
    tracing_handler_end()
}

#[poise::command(prefix_command, guild_only = true, check = "is_auth")]
#[instrument(name = "unranked-idea-reset", skip(ctx))]
/// Sets ideas back to the default
pub async fn reset(ctx: Context<'_>) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    do_ideas_reset(ctx, ctx.channel_id(), ctx.data()).await?;
    ctx.reply("Ideas reset completed").await?;
    tracing_handler_end()
}

#[instrument(skip(cache_http, data))]
pub async fn do_ideas_reset(
    cache_http: impl CacheHttp,
    channel_id: ChannelId,
    data: &Data,
) -> anyhow::Result<()> {
    info!("START");
    channel_id.say(&cache_http, "Ideas before reset").await?;
    display_ideas_channel(&cache_http, channel_id, data, true).await?;
    data.inner.unranked.ideas_reset()?;
    display_ideas_channel(&cache_http, channel_id, data, true).await?;
    tracing_handler_end()
}

#[instrument(skip(ctx))]
pub async fn display_ideas(ctx: &Context<'_>, is_verbose: bool) -> anyhow::Result<()> {
    info!("START");
    let builder = display_generate_reply(ctx, is_verbose).await?;
    ctx.send(builder).await?;
    tracing_handler_end()
}

#[instrument(skip(cache_http, data))]
pub async fn display_ideas_channel(
    cache_http: impl CacheHttp,
    channel_id: ChannelId,
    data: &Data,
    is_verbose: bool,
) -> anyhow::Result<()> {
    info!("START");
    let builder = display_generate_message(&cache_http, data, is_verbose).await?;
    channel_id.send_message(&cache_http, builder).await?;
    tracing_handler_end()
}

#[instrument(skip(ctx))]
pub async fn display_ideas_with_msg<S: Into<String> + Debug>(
    ctx: &Context<'_>,
    extra_msg: S,
) -> anyhow::Result<()> {
    info!("START");
    let builder = display_generate_reply(ctx, false).await?.content(extra_msg);
    ctx.send(builder).await?;
    tracing_handler_end()
}

async fn display_generate_reply(
    ctx: &Context<'_>,
    is_verbose: bool,
) -> anyhow::Result<CreateReply> {
    let embed = display_generate_embed(ctx, ctx.data(), is_verbose).await?;
    Ok(CreateReply::default().embed(embed))
}

async fn display_generate_message(
    cache_http: impl CacheHttp,
    data: &Data,
    is_verbose: bool,
) -> anyhow::Result<CreateMessage> {
    let embed = display_generate_embed(cache_http, data, is_verbose).await?;
    Ok(CreateMessage::new().embed(embed))
}

async fn display_generate_embed(
    cache_http: impl CacheHttp,
    data: &Data,
    is_verbose: bool,
) -> Result<CreateEmbed, anyhow::Error> {
    let ideas_as_string = data
        .inner
        .unranked
        .ideas_as_string(cache_http, is_verbose)
        .await?;
    let embed = CreateEmbed::new()
        .title(Ideas::DISPLAY_TITLE)
        .description(ideas_as_string);
    Ok(embed)
}

#[instrument(skip(ctx))]
async fn change_vote(ctx: Context<'_>, id: IdeaId, is_add_vote: bool) -> anyhow::Result<()> {
    info!("START");
    let was_change_made =
        ctx.data()
            .inner
            .unranked
            .idea_change_vote(id, ctx.author_id_number(), is_add_vote)?;
    display_ideas_with_msg(
        &ctx,
        format!(
            "Vote {}{} for Idea# {id}",
            if was_change_made { "" } else { "already " },
            if is_add_vote { "added" } else { "removed" }
        ),
    )
    .await?;
    tracing_handler_end()
}

#[instrument(skip(ctx))]
async fn change_vote_all(ctx: Context<'_>, is_add_vote: bool) -> anyhow::Result<()> {
    info!("START");
    let change_count = ctx
        .data()
        .inner
        .unranked
        .idea_change_vote_all(ctx.author_id_number(), is_add_vote)?;
    display_ideas_with_msg(
        &ctx,
        format!(
            "{change_count} Votes {}",
            if is_add_vote { "added" } else { "removed" }
        ),
    )
    .await?;
    tracing_handler_end()
}
