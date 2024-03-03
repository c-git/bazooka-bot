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
    pub channel_bot_status: Option<ChannelId>,
}

#[derive(Debug)]
pub struct SharedConfig {
    pub start_instant: Instant,
    pub auth_role_id: RoleId,
    pub persist: PersistInstance,
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
        let channel_bot_status = KeyName::ChannelBotStatus.get_non_secret_parse_opt(secret_store);

        Ok(Self {
            registration_guild_id: guild_id,
            owners,
            is_production,
            channel_bot_status,
        })
    }
}

impl SharedConfig {
    pub fn try_new(
        secret_store: &SecretStore,
        persist: PersistInstance,
    ) -> anyhow::Result<&'static Self> {
        let auth_role_id = KeyName::AuthRoleId.get_non_secret_parse(secret_store)?;
        let result = Box::new(Self {
            start_instant: Instant::now(),
            auth_role_id,
            persist,
        });
        Ok(Box::leak(result))
    }
}
