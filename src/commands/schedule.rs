//! Groups the commands related to scheduling

use std::num::NonZeroUsize;

use poise::{serenity_prelude::CreateEmbed, CreateReply};
use tracing::{info, instrument};

use crate::{
    commands::{call_to_parent_command, is_auth, tracing_handler_end, tracing_handler_start},
    model::schedule::{
        Objective, OutcomeCreateScheduledTask, ScheduledTaskId, ScheduledTasks, UnixTimestamp,
    },
    Context,
};

#[poise::command(
    prefix_command,
    slash_command,
    track_edits,
    subcommand_required,
    subcommands("set_unranked", "display", "cancel")
)]
#[instrument(name = "schedule", skip(ctx))]
/// Commands related to scheduling
pub async fn schedule(ctx: Context<'_>) -> anyhow::Result<()> {
    call_to_parent_command(ctx).await
}

#[poise::command(
    hide_in_help,
    prefix_command,
    slash_command,
    track_edits,
    guild_only = true,
    check = "is_auth"
)]
#[instrument(name = "schedule-set_unranked", skip(ctx))]
/// Sets when the next unranked is expected to start (use no args for more info)
pub async fn set_unranked(
    ctx: Context<'_>,
    #[description = "A unix timestamp. If you need more info just leave out argument for more info to be returned"]
    unix_timestamp: Option<i32>,
) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    if let Some(unix_timestamp) = unix_timestamp {
        let timestamp = UnixTimestamp::new(unix_timestamp);
        let outcome = ctx
            .data()
            .schedule_create_task(Objective::UnrankedStartEvent, timestamp)?;
        let mut msg = format!("Unranked Event Start Scheduled for {timestamp}");
        if let OutcomeCreateScheduledTask::Replaced(prev) = outcome {
            use std::fmt::Write as _;
            write!(msg, "\nCancelled previous schedule for {prev}")?;
        }
        ctx.reply(msg).await?;
    } else {
        info!("Info given, command not executed");
        ctx.reply(
            "This command expects a unix timestamp.
For help with generating a timestamp see <https://c-git.github.io/misc/discord/>
you can test your timestamp by pasting the string given on the site in discord.
Note: the command expects **ONLY** the number part",
        )
        .await?;
    }
    tracing_handler_end()
}

#[poise::command(prefix_command, slash_command, track_edits, aliases("disp"))]
#[instrument(name = "schedule-display", skip(ctx))]
/// Shows the scheduled tasks [aliases("disp")]
pub async fn display(ctx: Context<'_>) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    let tasks_as_string = ctx.data().schedule_as_string()?;
    let embed = CreateEmbed::new()
        .title(ScheduledTasks::DISPLAY_TITLE)
        .description(tasks_as_string);
    let builder = CreateReply::default().embed(embed);
    ctx.send(builder).await?;
    tracing_handler_end()
}

#[poise::command(
    hide_in_help,
    prefix_command,
    slash_command,
    track_edits,
    guild_only = true,
    check = "is_auth"
)]
#[instrument(name = "schedule-cancel", skip(ctx))]
/// Cancel a scheduled event
pub async fn cancel(
    ctx: Context<'_>,
    #[description = "See display to get valid values"] id: NonZeroUsize,
) -> anyhow::Result<()> {
    tracing_handler_start(&ctx).await;
    let id: ScheduledTaskId = id.into();
    let scheduled_task = ctx.data().schedule_cancel_task_by_id(id)?;
    ctx.reply(format!(
        "{} cancelled for {}",
        scheduled_task.objective, scheduled_task.desired_execution_timestamp
    ))
    .await?;
    tracing_handler_end()
}
