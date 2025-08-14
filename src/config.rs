use std::{collections::HashSet, fmt::Debug, time::Instant};

use anyhow::Context as _;
use poise::serenity_prelude::{ChannelId, GuildId, RoleId, UserId};
use shuttle_runtime::{SecretStore, tokio};
use tracing::error;

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
    pub db_pool: sqlx::PgPool,
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
        db_pool: sqlx::PgPool,
    ) -> anyhow::Result<&'static Self> {
        let auth_role_id = KeyName::AuthRoleId.get_non_secret_parse(secret_store)?;
        let channel_unranked = KeyName::ChannelUnrankedId.get_non_secret_parse(secret_store)?;
        let channel_bot_status = KeyName::ChannelBotStatus.get_non_secret_parse_opt(secret_store);
        let result = Box::new(Self {
            start_instant: Instant::now(),
            auth_role_id,
            channel_unranked,
            db_pool,
            channel_bot_status,
        });
        Ok(Box::leak(result))
    }

    /// Doesn't actually perform the save but spawns a task to do it in the background
    pub fn save_kv<T: serde::Serialize>(&self, key: &str, value: &T) -> anyhow::Result<()> {
        let pool = self.db_pool.clone();
        let key = key.to_string();
        let value = serde_json::to_string(value).context("failed to convert to json")?;
        tokio::spawn(async move {
            crate::db::save_kv(&pool, key, value).await;
        });
        Ok(())
    }

    pub async fn load_or_default_kv<T: serde::de::DeserializeOwned + Default>(
        &self,
        key: &str,
    ) -> T {
        let Some(content) = crate::db::load_kv(&self.db_pool, key).await else {
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
