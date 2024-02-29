//! Groups all the bot commands together. These then delegate to the model as needed

use tracing::info;

use crate::Context;

mod general;
mod unranked_cmd;

pub use self::{
    general::{help, ping},
    unranked_cmd::unranked,
};

/// Common info added to tracing for functions
fn tracing_handler_start(ctx: &Context) {
    info!("Author: {}", ctx.author().name);
}

/// Used to mark the end
fn tracing_handler_end() -> anyhow::Result<()> {
    info!("END");
    Ok(())
}

/// Standardized response to a call to a parent function (not callable by slash command)
async fn call_to_parent_command(ctx: Context<'_>) -> anyhow::Result<()> {
    info!("{} called a parent command", ctx.author().name);
    ctx.say("requires subcommand see /help").await?;
    Ok(())
}
