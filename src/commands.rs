//! Groups all the bot commands together. These then delegate to the model as needed

use tracing::{error, info};

use crate::{AuthorPreferredDisplay as _, Context, Data};

mod general;
mod unranked_cmd;

pub use self::{
    general::{help, ping},
    unranked_cmd::unranked,
};

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
    vec![ping(), help(), general::version(), unranked()]
}
