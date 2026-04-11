//! Cloudflare R2 storage client for lesson photo upload/delete.
//!
//! R2 is S3-compatible; we use the `rust-s3` crate for operations.
//! Photos are stored as `{user_id}/{lesson_id}/{photo_id}.jpg`.

use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::Region;

use crate::error::ApiError;

#[derive(Clone)]
pub struct R2Client {
    bucket: Box<Bucket>,
    public_url_base: String,
}

impl R2Client {
    /// Create a new R2 client from environment variables.
    ///
    /// Required env vars:
    /// - `R2_ACCOUNT_ID` — Cloudflare account ID
    /// - `R2_ACCESS_KEY_ID` — R2 API token access key
    /// - `R2_SECRET_ACCESS_KEY` — R2 API token secret key
    /// - `R2_BUCKET_NAME` — Name of the R2 bucket
    /// - `R2_PUBLIC_URL` — Public URL base for the bucket (e.g., `https://photos.example.com`)
    pub fn from_env() -> Result<Self, String> {
        let account_id =
            std::env::var("R2_ACCOUNT_ID").map_err(|_| "R2_ACCOUNT_ID must be set".to_string())?;
        let access_key = std::env::var("R2_ACCESS_KEY_ID")
            .map_err(|_| "R2_ACCESS_KEY_ID must be set".to_string())?;
        let secret_key = std::env::var("R2_SECRET_ACCESS_KEY")
            .map_err(|_| "R2_SECRET_ACCESS_KEY must be set".to_string())?;
        let bucket_name = std::env::var("R2_BUCKET_NAME")
            .map_err(|_| "R2_BUCKET_NAME must be set".to_string())?;
        let public_url =
            std::env::var("R2_PUBLIC_URL").map_err(|_| "R2_PUBLIC_URL must be set".to_string())?;

        let region = Region::Custom {
            region: "auto".to_string(),
            endpoint: format!("https://{account_id}.r2.cloudflarestorage.com"),
        };

        let credentials = Credentials::new(Some(&access_key), Some(&secret_key), None, None, None)
            .map_err(|e| format!("Failed to create R2 credentials: {e}"))?;

        let bucket = Bucket::new(&bucket_name, region, credentials)
            .map_err(|e| format!("Failed to create R2 bucket: {e}"))?
            .with_path_style();

        Ok(Self {
            bucket,
            public_url_base: public_url.trim_end_matches('/').to_string(),
        })
    }

    /// Generate the storage key for a lesson photo.
    pub fn photo_key(user_id: &str, lesson_id: &str, photo_id: &str) -> String {
        format!("{user_id}/{lesson_id}/{photo_id}.jpg")
    }

    /// Get the public URL for a storage key.
    pub fn public_url(&self, key: &str) -> String {
        format!("{}/{key}", self.public_url_base)
    }

    /// Upload photo bytes to R2.
    pub async fn upload(&self, key: &str, data: &[u8], content_type: &str) -> Result<(), ApiError> {
        self.bucket
            .put_object_with_content_type(key, data, content_type)
            .await
            .map_err(|e| ApiError::Internal(format!("R2 upload failed: {e}")))?;
        Ok(())
    }

    /// Delete a photo from R2.
    pub async fn delete(&self, key: &str) -> Result<(), ApiError> {
        self.bucket
            .delete_object(key)
            .await
            .map_err(|e| ApiError::Internal(format!("R2 delete failed: {e}")))?;
        Ok(())
    }

    /// Delete all photos for a lesson (by prefix).
    pub async fn delete_lesson_photos(
        &self,
        user_id: &str,
        lesson_id: &str,
    ) -> Result<(), ApiError> {
        let prefix = format!("{user_id}/{lesson_id}/");
        let list = self
            .bucket
            .list(prefix, None)
            .await
            .map_err(|e| ApiError::Internal(format!("R2 list failed: {e}")))?;

        for result in list {
            for obj in result.contents {
                self.bucket
                    .delete_object(&obj.key)
                    .await
                    .map_err(|e| ApiError::Internal(format!("R2 delete failed: {e}")))?;
            }
        }
        Ok(())
    }
}
