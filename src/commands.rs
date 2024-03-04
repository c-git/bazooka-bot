//! Groups all the bot commands together. These then delegate to the model as needed

use poise::serenity_prelude::Mentionable;
use tracing::{error, info, instrument, warn};

use crate::{
    commands::{
        general::{help, ping, register, uptime},
        schedule::schedule,
        unranked_cmd::unranked,
    },
    AuthorPreferredDisplay as _, Context, Data,
};

mod general;
mod schedule;
mod unranked_cmd;

/// Common info added to tracing for functions
async fn tracing_handler_start(ctx: &Context<'_>) {
    info!("Author name: {}", ctx.author().name);
    info!(
        "Author Display Name: {}",
        ctx.author_preferred_display().await
    );
}

/// Used to mark the end
fn tracing_handler_end() -> anyhow::Result<()> {
    info!("END");
    Ok(())
}

/// Standardized response to a call to a parent function (not callable by slash command)
async fn call_to_parent_command(ctx: Context<'_>) -> anyhow::Result<()> {
    error!(
        "Got a call to a parent command. Function needs to be annotated with `subcommand_required`. Called by {}",
        ctx.author().name
    );
    ctx.reply("requires subcommand see /help").await?;
    Ok(())
}

pub fn commands_list() -> Vec<poise::Command<Data, anyhow::Error>> {
    vec![
        ping(),
        help(),
        general::version(),
        uptime(),
        unranked(),
        schedule(),
        register(),
    ]
}

#[instrument(skip(ctx))]
async fn is_auth(ctx: Context<'_>) -> anyhow::Result<bool> {
    info!("START");
    let result;
    let role_id = ctx.data().inner.shared_config.auth_role_id;
    if let Some(member) = ctx.author_member().await {
        result = member.roles.contains(&role_id);
        if !result {
            ctx.reply(format!(
                "You don't have permission to run this command. Please see someone from {} for assistance",
                role_id.mention()
            ))
            .await?;
        }
    } else {
        result = false;
        warn!(
            "Unable to get membership for {:?}, likely sent in a DM.",
            ctx.author().name
        );
        ctx.say("This command is only allowed from a server")
            .await?;
    };
    if !result {
        warn!(
            "User: {:?} ({}) attempted to execute {:?} but they did not have role# {role_id}.",
            ctx.author().name,
            ctx.author().id,
            ctx.command().qualified_name,
        );
    }
    info!("END");
    Ok(result)
}
