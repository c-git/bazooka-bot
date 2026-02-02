use std::{collections::HashSet, fmt::Debug, time::Instant};

use anyhow::Context as _;
use poise::serenity_prelude::{ChannelId, GuildId, RoleId, UserId};
use tracing::error;

use crate::ClapConfig;

#[derive(Debug)]
pub struct StartupConfig {
    pub registration_guild_id: Option<GuildId>,
    pub owners: HashSet<UserId>,
    pub is_production: bool,
}

#[derive(Debug)]
/// Shares immutable data across various places in the application by each just having a pointer to a leaked instance of this struct
pub struct SharedConfig {
    pub start_instant: Instant,
    pub auth_role_id: RoleId,
    pub channel_unranked: ChannelId,
    pub channel_bot_status: Option<ChannelId>,
}

impl StartupConfig {
    pub fn try_new(clap_config: &ClapConfig) -> anyhow::Result<Self> {
        let guild_id = clap_config
            .registration_guild_id
            .parse::<u64>()
            .context("failed to parse guild id")
            .map(GuildId::new)?;

        // TODO 4 - See if we can split this in clap
        let owners: HashSet<UserId> = clap_config
            .owners
            .split(',')
            .map(|x| {
                x.parse::<u64>()
                    .context("failed to parse owner")
                    .map(UserId::new)
            })
            .collect::<anyhow::Result<HashSet<UserId>>>()?;

        let is_production = std::env::var("SHUTTLE").is_ok();

        Ok(dbg!(Self {
            registration_guild_id: Some(guild_id),
            owners,
            is_production,
        }))
    }
}

impl SharedConfig {
    pub fn try_new(clap_config: &ClapConfig) -> anyhow::Result<&'static Self> {
        let auth_role_id = clap_config
            .auth_role_id
            .parse::<u64>()
            .context("failed to parse auth role id")
            .map(RoleId::new)?;
        let channel_unranked = clap_config
            .channel_unranked_id
            .parse::<u64>()
            .context("failed to parse unranked channel id")
            .map(ChannelId::new)?;
        let channel_bot_status = clap_config
            .channel_bot_status_id
            .parse::<u64>()
            .context("failed to parse bot status channel id")
            .ok()
            .map(ChannelId::new);
        let result = Box::new(dbg!(Self {
            start_instant: Instant::now(),
            auth_role_id,
            channel_unranked,
            channel_bot_status,
        }));
        Ok(Box::leak(result))
    }

    /// Doesn't actually perform the save but spawns a task to do it in the background
    pub fn save_kv<T: serde::Serialize>(&self, key: &str, value: &T) -> anyhow::Result<()> {
        let key = key.to_string();
        let value = serde_json::to_string(value).context("failed to convert to json")?;
        tokio::spawn(async move {
            crate::db::save_kv(&key, value).await;
        });
        Ok(())
    }

    pub async fn load_or_default_kv<T: serde::de::DeserializeOwned + Default>(
        &self,
        key: &str,
    ) -> T {
        let Some(content) = crate::db::load_kv(key).await else {
            return T::default();
        };
        match serde_json::from_str(&content) {
            Ok(x) => x,
            Err(err_msg) => {
                error!(
                    ?err_msg,
                    ?content,
                    "Failed to convert content extracted from the database"
                );
                T::default()
            }
        }
    }
}
