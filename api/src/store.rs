use eyre::Result;
use worker::kv::KvStore;

#[allow(async_fn_in_trait)]
pub trait KvWrapper {
    async fn put_val(&mut self, key: &str, value: &str) -> Result<()>;
    async fn get_val(&self, key: &str) -> Result<Option<String>>;
    async fn delete_val(&mut self, key: &str) -> Result<()>;
}

impl KvWrapper for KvStore {
    async fn put_val(&mut self, key: &str, value: &str) -> Result<()> {
        self.put(key, value)
            .map_err(|e| eyre::eyre!("Failed to put key: {}", e))?
            .execute()
            .await
            .or_else(|e| eyre::bail!("Failed to put key: {}", e))?;

        Ok(())
    }

    async fn get_val(&self, key: &str) -> Result<Option<String>> {
        let res = match self.get(key).text().await {
            Ok(value) => value,
            Err(e) => {
                eyre::bail!("Failed to get key: {}", e)
            }
        };

        Ok(res)
    }

    async fn delete_val(&mut self, key: &str) -> Result<()> {
        self.delete(key)
            .await
            .or_else(|e| eyre::bail!("Failed to delete key: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    pub struct MockKvStore {
        pub store: HashMap<String, String>,
    }

    impl KvWrapper for MockKvStore {
        async fn put_val(&mut self, key: &str, value: &str) -> Result<()> {
            self.store.insert(key.to_string(), value.to_string());
            Ok(())
        }

        async fn get_val(&self, key: &str) -> Result<Option<String>> {
            Ok(self.store.get(key).map(|v| v.to_string()))
        }

        async fn delete_val(&mut self, key: &str) -> Result<()> {
            self.store.remove(key);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_kv_wrapper() -> Result<()> {
        let mut kv = MockKvStore::default();

        let empty = kv.get_val("test").await?;
        assert!(empty.is_none());

        kv.put_val("test_key", "test_val").await?;

        let present = kv.get_val("test_key").await?;
        assert_eq!(present, Some("test_val".to_string()));

        Ok(())
    }
}
