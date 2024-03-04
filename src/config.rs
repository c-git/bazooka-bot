use std::{collections::HashSet, time::Instant};

use anyhow::Context as _;
use poise::serenity_prelude::{ChannelId, GuildId, RoleId, UserId};
use shuttle_persist::PersistInstance;
use shuttle_secrets::SecretStore;

use crate::secrets::KeyName;

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
    pub persist: PersistInstance,
    pub channel_bot_status: Option<ChannelId>,
}

impl StartupConfig {
    pub fn try_new(secret_store: &SecretStore) -> anyhow::Result<Self> {
        let guild_id = KeyName::RegistrationGuildId.get_non_secret_parse_opt(secret_store);

        let owners: HashSet<UserId> = KeyName::Owners
            .get_non_secret_string(secret_store)?
            .split(',')
            .map(|x| {
                x.parse::<u64>()
                    .context("failed to parse owner")
                    .map(UserId::new)
            })
            .collect::<anyhow::Result<HashSet<UserId>>>()?;

        let is_production = std::env::var("SHUTTLE").is_ok();

        Ok(Self {
            registration_guild_id: guild_id,
            owners,
            is_production,
        })
    }
}

impl SharedConfig {
    pub fn try_new(
        secret_store: &SecretStore,
        persist: PersistInstance,
    ) -> anyhow::Result<&'static Self> {
        let auth_role_id = KeyName::AuthRoleId.get_non_secret_parse(secret_store)?;
        let channel_unranked = KeyName::ChannelUnrankedId.get_non_secret_parse(secret_store)?;
        let channel_bot_status = KeyName::ChannelBotStatus.get_non_secret_parse_opt(secret_store);
        let result = Box::new(Self {
            start_instant: Instant::now(),
            auth_role_id,
            channel_unranked,
            persist,
            channel_bot_status,
        });
        Ok(Box::leak(result))
    }
}
