//! The commands related to the idea functionality for unranked

use std::num::NonZeroUsize;

use poise::serenity_prelude::User;
use tracing::instrument;

use crate::{
    commands::{call_to_parent_command, fn_start_tracing, Context},
    model::{
        unranked::{Idea, IdeaId},
        Data,
    },
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
pub async fn add(ctx: Context<'_>, #[rest] description: String) -> anyhow::Result<()> {
    fn_start_tracing(&ctx);
    ctx.data().add(ctx.author(), description)?;
    display_ideas_with_msg(&ctx, "Idea Added").await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command, track_edits)]
#[instrument(name = "unranked-idea-edit", skip(ctx))]
/// Edits an idea you previously created
pub async fn edit(
    ctx: Context<'_>,
    id: NonZeroUsize,
    #[rest] new_description: String,
) -> anyhow::Result<()> {
    fn_start_tracing(&ctx);
    let id: IdeaId = id.into();
    ctx.data().edit(id, ctx.author(), new_description)?;
    display_ideas_with_msg(&ctx, "Idea Updated").await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
#[instrument(name = "unranked-idea-remove", skip(ctx))]
/// Removes and idea you previously created
pub async fn remove(ctx: Context<'_>, id: NonZeroUsize) -> anyhow::Result<()> {
    fn_start_tracing(&ctx);
    let id: IdeaId = id.into();
    let old_idea = ctx.data().remove(id, ctx.author())?;
    display_ideas_with_msg(
        &ctx,
        format!(
            "Idea Removed. It was ID {id} with description: {:?}",
            old_idea.description()
        ),
    )
    .await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
#[instrument(name = "unranked-idea-vote", skip(ctx))]
/// Adds your vote for the given idea (If you are currently voting for it nothing happens)
pub async fn vote(ctx: Context<'_>, id: NonZeroUsize) -> anyhow::Result<()> {
    fn_start_tracing(&ctx);
    todo!()
}

#[poise::command(prefix_command, slash_command)]
#[instrument(name = "unranked-idea-unvote", skip(ctx))]
/// Removes your vote for the given idea (If you are not currently voting for it nothing happens)
pub async fn unvote(ctx: Context<'_>, id: NonZeroUsize) -> anyhow::Result<()> {
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
/// Displays all ideas optionally verbosely
pub async fn display(ctx: Context<'_>, #[flag] is_verbose: bool) -> anyhow::Result<()> {
    fn_start_tracing(&ctx);
    display_ideas(&ctx, is_verbose).await
}

pub async fn display_ideas(ctx: &Context<'_>, is_verbose: bool) -> anyhow::Result<()> {
    let ideas_as_string = ctx.data().unranked_ideas_as_string(ctx, is_verbose).await?;
    ctx.say(ideas_as_string).await?;
    Ok(())
}

pub async fn display_ideas_with_msg<S: Into<String>>(
    ctx: &Context<'_>,
    extra_msg: S,
) -> anyhow::Result<()> {
    display_ideas(ctx, false).await?;
    ctx.reply(extra_msg).await?;
    Ok(())
}

impl Data {
    fn add(&self, user: &User, description: String) -> anyhow::Result<()> {
        self.unranked_idea_add(user, description)
    }

    fn edit(&self, id: IdeaId, user: &User, new_description: String) -> anyhow::Result<()> {
        self.unranked_idea_edit(id, user, new_description)
    }

    /// Attempts to remove and return the Idea
    fn remove(&self, id: IdeaId, user: &User) -> anyhow::Result<Idea> {
        self.unranked_idea_remove(id, user)
    }
}
