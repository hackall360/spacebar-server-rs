use std::path::{Path, PathBuf};

use anyhow::Result;
use async_trait::async_trait;

/// Abstraction over attachment storage backends.
#[async_trait]
pub trait Storage: Send + Sync {
    async fn set(&self, path: &str, data: &[u8]) -> Result<()>;
    async fn get(&self, path: &str) -> Result<Option<Vec<u8>>>;
    async fn delete(&self, path: &str) -> Result<()>;
}

/// Storage backend that keeps files on the local filesystem.
pub struct LocalStorage {
    root: PathBuf,
}

impl LocalStorage {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn resolve(&self, path: &str) -> PathBuf {
        self.root.join(Path::new(path))
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn set(&self, path: &str, data: &[u8]) -> Result<()> {
        let full = self.resolve(path);
        if let Some(parent) = full.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(full, data).await?;
        Ok(())
    }

    async fn get(&self, path: &str) -> Result<Option<Vec<u8>>> {
        let full = self.resolve(path);
        match tokio::fs::read(full).await {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let full = self.resolve(path);
        match tokio::fs::remove_file(&full).await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

/// Build a storage backend based on environment variables.
pub async fn build_storage() -> Result<ArcStorage> {
    let provider = std::env::var("STORAGE_PROVIDER").unwrap_or_else(|_| "file".into());

    if provider == "s3" {
        #[cfg(feature = "s3")]
        {
            return Ok(ArcStorage::new(Box::new(S3Storage::new_from_env().await?)));
        }

        // If S3 feature is not enabled fallback to local storage.
    }

    let root = std::env::var("STORAGE_LOCATION").unwrap_or_else(|_| "files".into());
    Ok(ArcStorage::new(Box::new(LocalStorage::new(PathBuf::from(
        root,
    )))))
}

/// Wrapper around a boxed trait object so it can be cloned.
#[derive(Clone)]
pub struct ArcStorage(std::sync::Arc<dyn Storage>);

impl ArcStorage {
    pub fn new(inner: Box<dyn Storage>) -> Self {
        Self(std::sync::Arc::from(inner))
    }

    pub fn inner(&self) -> &dyn Storage {
        &*self.0
    }
}

impl std::ops::Deref for ArcStorage {
    type Target = dyn Storage;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

#[cfg(feature = "s3")]
mod s3 {
    use super::Storage;
    use anyhow::Result;
    use async_trait::async_trait;
    use aws_sdk_s3::{types::ByteStream, Client, Region};

    pub struct S3Storage {
        client: Client,
        bucket: String,
        prefix: Option<String>,
    }

    impl S3Storage {
        pub async fn new_from_env() -> Result<Self> {
            let region = std::env::var("STORAGE_REGION").expect("STORAGE_REGION missing");
            let bucket = std::env::var("STORAGE_BUCKET").expect("STORAGE_BUCKET missing");
            let prefix = std::env::var("STORAGE_LOCATION").ok();

            let conf = aws_config::from_env()
                .region(Region::new(region))
                .load()
                .await;
            let client = Client::new(&conf);

            Ok(Self {
                client,
                bucket,
                prefix,
            })
        }

        fn key(&self, path: &str) -> String {
            match &self.prefix {
                Some(p) => format!("{}{}", p, path),
                None => path.to_string(),
            }
        }
    }

    #[async_trait]
    impl Storage for S3Storage {
        async fn set(&self, path: &str, data: &[u8]) -> Result<()> {
            self.client
                .put_object()
                .bucket(&self.bucket)
                .key(self.key(path))
                .body(ByteStream::from(data.to_owned()))
                .send()
                .await?;
            Ok(())
        }

        async fn get(&self, path: &str) -> Result<Option<Vec<u8>>> {
            match self
                .client
                .get_object()
                .bucket(&self.bucket)
                .key(self.key(path))
                .send()
                .await
            {
                Ok(obj) => {
                    let data = obj.body.collect().await?.into_bytes().to_vec();
                    Ok(Some(data))
                }
                Err(err) => {
                    if err.is_not_found() {
                        Ok(None)
                    } else {
                        Err(err.into())
                    }
                }
            }
        }

        async fn delete(&self, path: &str) -> Result<()> {
            self.client
                .delete_object()
                .bucket(&self.bucket)
                .key(self.key(path))
                .send()
                .await?;
            Ok(())
        }
    }

    pub use S3Storage;
}

#[cfg(feature = "s3")]
pub use s3::S3Storage;
