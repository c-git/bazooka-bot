//! Needed because the types used by poise are not able to be deserialized from Bincode

use std::fmt::Display;

use poise::serenity_prelude::{CacheHttp, User, UserId};

use crate::{AuthorPreferredDisplay as _, Context};

/// Created to use in place of User or UserId from Framework because they
/// are not able to be deserialized from Bincode which shuttle-persist uses
#[derive(Debug, serde::Serialize, serde::Deserialize, Default, Clone, Copy, PartialEq, Eq)]
pub struct UserIdNumber(u64);

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone)]
pub struct UserName(String);

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone)]
pub struct UserRecord {
    pub id_number: UserIdNumber,
    pub name: UserName,
}

pub trait UserRecordSupport {
    fn author_id_number(&self) -> UserIdNumber;
    async fn author_to_user_record(&self) -> UserRecord;
}

impl UserRecord {
    pub async fn from_author(ctx: &Context<'_>) -> Self {
        let id_number = ctx.author().id.into();
        let name = ctx.author_preferred_display().await.into();
        Self { id_number, name }
    }
}

impl UserIdNumber {
    pub async fn to_user(self, cache_http: impl CacheHttp) -> anyhow::Result<User> {
        Ok(self.to_user_id().to_user(cache_http).await?)
    }

    pub fn to_user_id(self) -> UserId {
        UserId::from(self.0)
    }
}

impl<T: AsRef<UserId>> From<T> for UserIdNumber {
    fn from(value: T) -> Self {
        Self(value.as_ref().get())
    }
}

impl Display for UserName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<S: Into<String>> From<S> for UserName {
    fn from(value: S) -> Self {
        Self(value.into())
    }
}

impl UserRecordSupport for Context<'_> {
    fn author_id_number(&self) -> UserIdNumber {
        self.author().id.into()
    }

    async fn author_to_user_record(&self) -> UserRecord {
        UserRecord::from_author(self).await
    }
}

impl Display for UserIdNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
