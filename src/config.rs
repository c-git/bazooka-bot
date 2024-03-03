use std::{collections::HashSet, time::Instant};

use anyhow::Context as _;
use poise::serenity_prelude::{GuildId, RoleId, UserId};
use shuttle_persist::PersistInstance;
use shuttle_secrets::SecretStore;

use crate::{secrets::AccessSecrets as _, KeyName};

#[derive(Debug)]
pub struct StartupConfig {
    pub guild_id: GuildId,
    pub owners: HashSet<UserId>,
    pub is_production: bool,
}

#[derive(Debug)]
pub struct SharedConfig {
    pub start_instant: Instant,
    pub auth_role_id: RoleId,
    pub persist: PersistInstance,
}

impl StartupConfig {
    pub fn try_new(secret_store: &SecretStore) -> anyhow::Result<Self> {
        let guild_id = KeyName::GuildId.get_stored_non_secret_parse(secret_store)?;

        let owners: HashSet<UserId> = KeyName::Owners
            .get_stored_non_secret_string(secret_store)?
            .split(',')
            .map(|x| {
                x.parse::<u64>()
                    .context("failed to parse owner")
                    .map(UserId::new)
            })
            .collect::<anyhow::Result<HashSet<UserId>>>()?;

        let is_production = std::env::var("SHUTTLE").is_ok();

        Ok(Self {
            guild_id,
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
        let auth_role_id = secret_store.access_secret_parse("AUTH_ROLE_ID")?;
        let result = Box::new(Self {
            start_instant: Instant::now(),
            auth_role_id,
            persist,
        });
        Ok(Box::leak(result))
    }
}
