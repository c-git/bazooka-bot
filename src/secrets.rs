//! This modules handles access to all the secret information

use std::str::FromStr;

use anyhow::{bail, Context as _};
use secrecy::{zeroize::DefaultIsZeroes, Secret, SecretString};
use shuttle_secrets::SecretStore;

pub enum KeyName {
    DiscordToken,
    GuildId,
    AuthRoleId,
    Owners,
    ChannelUnrankedId,
    ChannelAdminId,
    ChannelBotStatus,
}

impl AsRef<str> for KeyName {
    fn as_ref(&self) -> &str {
        match self {
            KeyName::DiscordToken => "DISCORD_TOKEN",
            KeyName::GuildId => "GUILD_ID",
            KeyName::AuthRoleId => "AUTH_ROLE_ID",
            KeyName::Owners => "OWNERS",
            KeyName::ChannelUnrankedId => "CHANNEL_UNRANKED_ID",
            KeyName::ChannelAdminId => "CHANNEL_ADMIN_ID",
            KeyName::ChannelBotStatus => "CHANNEL_BOT_STATUS_ID",
        }
    }
}

impl KeyName {
    pub fn get_stored_secret_string(
        &self,
        secret_store: &SecretStore,
    ) -> anyhow::Result<SecretString> {
        Ok(SecretString::new(
            secret_store.access_secret_string(self.as_ref())?,
        ))
    }
    pub fn get_stored_secret_parse<F: FromStr + DefaultIsZeroes>(
        &self,
        secret_store: &SecretStore,
    ) -> anyhow::Result<Secret<F>> {
        Ok(Secret::new(
            secret_store.access_secret_parse(self.as_ref())?,
        ))
    }
    pub fn get_stored_non_secret_string(
        &self,
        secret_store: &SecretStore,
    ) -> anyhow::Result<String> {
        Ok(secret_store.access_secret_string(self.as_ref())?)
    }
    pub fn get_stored_non_secret_parse<F: FromStr>(
        &self,
        secret_store: &SecretStore,
    ) -> anyhow::Result<F> {
        Ok(secret_store.access_secret_parse(self.as_ref())?)
    }
}

pub trait AccessSecrets {
    fn access_secret_parse<F: FromStr>(&self, key: &str) -> anyhow::Result<F>;
    fn access_secret_string(&self, key: &str) -> anyhow::Result<String>;
}
impl AccessSecrets for SecretStore {
    fn access_secret_parse<F: FromStr>(&self, key: &str) -> anyhow::Result<F> {
        let value = self.access_secret_string(key)?;
        match value.parse() {
            Ok(result) => Ok(result),
            Err(_) => bail!("failed to parse {key}. Value: '{value}'"),
        }
    }

    fn access_secret_string(&self, key: &str) -> anyhow::Result<String> {
        self.get(key)
            .with_context(|| format!("'{key}' was not found"))
    }
}
