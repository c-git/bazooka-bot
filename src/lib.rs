//! This is the library for the application. The majority of the logic can be found here
//! It is split into two main parts. The parts that receive commands from discord [`commands`] and
//! the part that handles the actual logic of what to do in the [`model`]

pub mod commands;
mod model;

pub use model::Data;

/// Type used by poise framework as the context when commands are triggered
type Context<'a> = poise::Context<'a, Data, anyhow::Error>;
