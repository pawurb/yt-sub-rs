use chrono::{DateTime, Utc};
use eyre::Result;

use crate::store::KvWrapper;
static USER_IDS_KEY: &str = "user_ids";

#[derive(Debug, PartialEq)]
pub struct User {
    pub api_key: String,
}

impl User {
    pub fn new(api_key: &str) -> User {
        User {
            api_key: api_key.to_string(),
        }
    }

    pub async fn list(kv: &impl KvWrapper) -> Result<Vec<User>> {
        let user_ids = kv
            .get_val(USER_IDS_KEY)
            .await?
            .unwrap_or_else(|| "".to_string());

        let user_ids: Vec<&str> = user_ids.split(',').filter(|s| !s.is_empty()).collect();

        Ok(user_ids.into_iter().map(User::new).collect())
    }

    pub async fn save(user: &User, kv: &mut impl KvWrapper) -> Result<()> {
        let user_ids = kv
            .get_val(USER_IDS_KEY)
            .await?
            .unwrap_or_else(|| "".to_string());

        let mut user_ids: Vec<&str> = user_ids.split(',').filter(|s| !s.is_empty()).collect();

        if user_ids.contains(&user.api_key.as_str()) {
            eyre::bail!("Already exists user with API key: {}", user.api_key)
        } else {
            user_ids.push(&user.api_key);
            kv.put_val(USER_IDS_KEY, &user_ids.join(",")).await?;
        }

        Ok(())
    }

    pub async fn last_run_at(&self, kv: &impl KvWrapper) -> Result<Option<DateTime<Utc>>> {
        let key = format!("last_run_at_{}", self.api_key);

        let Some(last_run_at) = kv.get_val(&key).await? else {
            return Ok(None);
        };
        let last_run_at = DateTime::parse_from_rfc3339(&last_run_at)
            .expect("Failed to parse last_run_at file")
            .with_timezone(&Utc);

        Ok(Some(last_run_at))
    }

    pub async fn touch_last_run_at(&self, kv: &mut impl KvWrapper) -> Result<()> {
        let key = format!("last_run_at_{}", self.api_key);
        kv.put_val(&key, &Utc::now().to_rfc3339()).await?;

        Ok(())
    }
}

#[cfg(test)]

mod tests {
    use crate::store::tests::MockKvStore;

    use super::*;

    #[tokio::test]
    async fn test_add_list_users() -> Result<()> {
        let mut kv = MockKvStore::default();

        let before = User::list(&kv).await?;
        assert_eq!(before.len(), 0);

        let new_user = User {
            api_key: "test".to_string(),
        };

        User::save(&new_user, &mut kv).await?;

        let after = User::list(&kv).await?;

        assert_eq!(after.len(), 1);
        assert_eq!(after[0], new_user);

        Ok(())
    }

    #[tokio::test]
    async fn test_last_run_at() -> Result<()> {
        let user = User::new("test");
        let mut kv = MockKvStore::default();

        assert_eq!(user.last_run_at(&kv).await?, None);

        user.touch_last_run_at(&mut kv).await?;

        assert!(user.last_run_at(&kv).await?.is_some());
        Ok(())
    }
}
