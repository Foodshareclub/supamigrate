use crate::error::{Result, SupamigrateError};
use bytes::Bytes;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct StorageClient {
    client: Client,
    api_url: String,
    service_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bucket {
    pub id: String,
    pub name: String,
    pub public: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageObject {
    pub name: String,
    pub id: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateBucketRequest {
    name: String,
    public: bool,
}

impl StorageClient {
    pub fn new(api_url: String, service_key: String) -> Self {
        Self {
            client: Client::new(),
            api_url,
            service_key,
        }
    }

    fn storage_url(&self) -> String {
        format!("{}/storage/v1", self.api_url)
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.service_key)
    }

    /// List all buckets
    pub async fn list_buckets(&self) -> Result<Vec<Bucket>> {
        let url = format!("{}/bucket", self.storage_url());
        debug!("Listing buckets: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("apikey", &self.service_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(SupamigrateError::Storage(format!(
                "Failed to list buckets: {} - {}",
                status, body
            )));
        }

        let buckets: Vec<Bucket> = response.json().await?;
        Ok(buckets)
    }

    /// Create a bucket
    pub async fn create_bucket(&self, name: &str, public: bool) -> Result<()> {
        let url = format!("{}/bucket", self.storage_url());
        debug!("Creating bucket: {}", name);

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("apikey", &self.service_key)
            .json(&CreateBucketRequest {
                name: name.to_string(),
                public,
            })
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            // Ignore "already exists" errors
            if !body.contains("already exists") {
                return Err(SupamigrateError::Storage(format!(
                    "Failed to create bucket '{}': {} - {}",
                    name, status, body
                )));
            }
        }

        Ok(())
    }

    /// List objects in a bucket (single page)
    async fn list_objects_page(
        &self,
        bucket: &str,
        prefix: &str,
        offset: u64,
    ) -> Result<Vec<StorageObject>> {
        let url = format!("{}/object/list/{}", self.storage_url(), bucket);

        let body = serde_json::json!({
            "prefix": prefix,
            "limit": 1000,
            "offset": offset,
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("apikey", &self.service_key)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(SupamigrateError::Storage(format!(
                "Failed to list objects in '{}': {} - {}",
                bucket, status, body
            )));
        }

        let objects: Vec<StorageObject> = response.json().await?;
        Ok(objects)
    }

    /// List all objects in a bucket recursively with pagination
    pub async fn list_objects(
        &self,
        bucket: &str,
        prefix: Option<&str>,
    ) -> Result<Vec<StorageObject>> {
        let prefix_str = prefix.unwrap_or("");
        debug!("Listing objects in bucket: {} prefix: {:?}", bucket, prefix_str);

        let mut all_files: Vec<StorageObject> = Vec::new();
        let mut folders_to_scan: Vec<String> = vec![prefix_str.to_string()];

        while let Some(folder) = folders_to_scan.pop() {
            let mut offset = 0u64;
            loop {
                let objects = self.list_objects_page(bucket, &folder, offset).await?;
                let count = objects.len();

                for obj in objects {
                    if obj.id.is_some() {
                        // It's a file — build full path
                        all_files.push(StorageObject {
                            name: format!("{}{}", folder, obj.name),
                            id: obj.id,
                            metadata: obj.metadata,
                            created_at: obj.created_at,
                            updated_at: obj.updated_at,
                        });
                    } else {
                        // It's a folder — queue for recursive scan
                        folders_to_scan.push(format!("{}{}/", folder, obj.name));
                    }
                }

                if count < 1000 {
                    break;
                }
                offset += 1000;
            }
        }

        debug!("Found {} files in bucket {}", all_files.len(), bucket);
        Ok(all_files)
    }

    /// Download an object, returning bytes and content type
    pub async fn download(&self, bucket: &str, path: &str) -> Result<(Bytes, String)> {
        let url = format!("{}/object/{}/{}", self.storage_url(), bucket, path);
        debug!("Downloading: {}/{}", bucket, path);

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("apikey", &self.service_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(SupamigrateError::Storage(format!(
                "Failed to download '{}/{}': {} - {}",
                bucket, path, status, body
            )));
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_else(|| mime_from_path(path))
            .to_string();

        let bytes = response.bytes().await?;
        Ok((bytes, content_type))
    }

    /// Upload an object with a specific content type
    pub async fn upload(&self, bucket: &str, path: &str, data: Bytes, content_type: &str) -> Result<()> {
        let url = format!("{}/object/{}/{}", self.storage_url(), bucket, path);
        debug!("Uploading: {}/{} ({})", bucket, path, content_type);

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("apikey", &self.service_key)
            .header("Content-Type", content_type)
            .header("x-upsert", "true")
            .body(data)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(SupamigrateError::Storage(format!(
                "Failed to upload '{}/{}': {} - {}",
                bucket, path, status, body
            )));
        }

        Ok(())
    }
}

/// Guess MIME type from file extension
pub(crate) fn mime_from_path(path: &str) -> &str {
    match path.rsplit('.').next().map(|s| s.to_lowercase()).as_deref() {
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("pdf") => "application/pdf",
        Some("json") => "application/json",
        Some("txt") => "text/plain",
        Some("html" | "htm") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("heic") => "image/heic",
        Some("heif") => "image/heif",
        Some("avif") => "image/avif",
        _ => "application/octet-stream",
    }
}
