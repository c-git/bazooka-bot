//! The commands related to the idea functionality for unranked

use poise::serenity_prelude::User;
use tracing::instrument;

use crate::{
    commands::{call_to_parent_command, fn_start_tracing, Context},
    model::Data,
};

#[poise::command(
    prefix_command,
    slash_command,
    track_edits,
    subcommands(
        "add",
        "edit",
        "remove",
        "vote",
        "unvote",
        "vote_all",
        "unvote_all",
        "display"
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
pub async fn add(ctx: Context<'_>, description: String) -> anyhow::Result<()> {
    fn_start_tracing(&ctx);
    ctx.data().add(ctx.author(), description).await?;
    display_ideas(&ctx, Some("Idea Saved")).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command, track_edits)]
#[instrument(name = "unranked-idea-edit", skip(ctx))]
/// Edits an idea you previously created
// TODO 4: Replace u32 with IdeaID
pub async fn edit(ctx: Context<'_>, id: u32, new_description: String) -> anyhow::Result<()> {
    fn_start_tracing(&ctx);
    todo!()
}

#[poise::command(prefix_command, slash_command)]
#[instrument(name = "unranked-idea-remove", skip(ctx))]
/// Removes and idea you previously created
// TODO 4: Replace u32 with IdeaID
pub async fn remove(ctx: Context<'_>, id: u32) -> anyhow::Result<()> {
    fn_start_tracing(&ctx);
    todo!()
}

#[poise::command(prefix_command, slash_command)]
#[instrument(name = "unranked-idea-vote", skip(ctx))]
/// Adds your vote for the given idea (If you are currently voting for it nothing happens)
// TODO 4: Replace u32 with IdeaID
pub async fn vote(ctx: Context<'_>, id: u32) -> anyhow::Result<()> {
    fn_start_tracing(&ctx);
    todo!()
}

#[poise::command(prefix_command, slash_command)]
#[instrument(name = "unranked-idea-unvote", skip(ctx))]
/// Removes your vote for the given idea (If you are not currently voting for it nothing happens)
// TODO 4: Replace u32 with IdeaID
pub async fn unvote(ctx: Context<'_>, id: u32) -> anyhow::Result<()> {
    fn_start_tracing(&ctx);
    todo!()
}

#[poise::command(prefix_command, slash_command)]
#[instrument(name = "unranked-idea-vote_all", skip(ctx))]
/// Adds your vote for all current ideas
pub async fn vote_all(ctx: Context<'_>) -> anyhow::Result<()> {
    fn_start_tracing(&ctx);
    todo!()
}

#[poise::command(prefix_command, slash_command)]
#[instrument(name = "unranked-idea-unvote_all", skip(ctx))]
/// Removes your vote for all ideas
pub async fn unvote_all(ctx: Context<'_>) -> anyhow::Result<()> {
    fn_start_tracing(&ctx);
    todo!()
}

#[poise::command(prefix_command, slash_command, track_edits)]
#[instrument(name = "unranked-idea-unvote_all", skip(ctx))]
/// Removes your vote for all ideas
pub async fn display(ctx: Context<'_>) -> anyhow::Result<()> {
    fn_start_tracing(&ctx);
    display_ideas::<&str>(&ctx, None).await
}

pub async fn display_ideas<S: Into<String>>(
    ctx: &Context<'_>,
    extra_msg: Option<S>,
) -> anyhow::Result<()> {
    ctx.say("Show ideas here").await?; // TODO 1 : Continue here
    if let Some(s) = extra_msg {
        ctx.reply(s).await?;
    }
    Ok(())
}

impl Data {
    async fn add(&self, user: &User, description: String) -> anyhow::Result<()> {
        self.unranked_idea_add(user, description)
    }
}
