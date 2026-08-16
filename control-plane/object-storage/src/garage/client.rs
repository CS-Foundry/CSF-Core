use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct GarageClient {
    http: reqwest::Client,
    admin_url: String,
    admin_token: String,
}

#[derive(Debug, Deserialize)]
pub struct GarageBucket {
    pub id: String,
}

#[derive(Debug, Serialize)]
struct CreateBucketBody {
    #[serde(rename = "globalAlias")]
    global_alias: String,
}

#[derive(Debug, Serialize)]
struct UpdateBucketBody {
    quotas: UpdateBucketQuotas,
}

#[derive(Debug, Serialize)]
struct UpdateBucketQuotas {
    #[serde(rename = "maxSize", skip_serializing_if = "Option::is_none")]
    max_size: Option<i64>,
    #[serde(rename = "maxObjects", skip_serializing_if = "Option::is_none")]
    max_objects: Option<i64>,
}

impl GarageClient {
    pub fn new(admin_url: String, admin_token: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            admin_url,
            admin_token,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.admin_url, path)
    }

    async fn check_status(response: reqwest::Response, context: &str) -> Result<reqwest::Response> {
        if response.status().is_success() {
            return Ok(response);
        }
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        bail!("garage admin api error context={} status={} body={}", context, status, body)
    }

    pub async fn create_bucket(&self, global_alias: &str) -> Result<GarageBucket> {
        let response = self
            .http
            .post(self.url("/v2/CreateBucket"))
            .bearer_auth(&self.admin_token)
            .json(&CreateBucketBody {
                global_alias: global_alias.to_string(),
            })
            .send()
            .await
            .context("create_bucket request failed")?;

        let response = Self::check_status(response, "create_bucket").await?;
        response
            .json::<GarageBucket>()
            .await
            .context("failed to parse create_bucket response")
    }

    pub async fn update_bucket_quotas(
        &self,
        garage_bucket_id: &str,
        max_size: Option<i64>,
        max_objects: Option<i64>,
    ) -> Result<()> {
        let response = self
            .http
            .post(self.url(&format!("/v2/UpdateBucket?id={}", garage_bucket_id)))
            .bearer_auth(&self.admin_token)
            .json(&UpdateBucketBody {
                quotas: UpdateBucketQuotas {
                    max_size,
                    max_objects,
                },
            })
            .send()
            .await
            .context("update_bucket_quotas request failed")?;

        Self::check_status(response, "update_bucket_quotas").await?;
        Ok(())
    }

    pub async fn delete_bucket(&self, garage_bucket_id: &str) -> Result<()> {
        let response = self
            .http
            .delete(self.url(&format!("/v2/DeleteBucket?id={}", garage_bucket_id)))
            .bearer_auth(&self.admin_token)
            .send()
            .await
            .context("delete_bucket request failed")?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(());
        }

        Self::check_status(response, "delete_bucket").await?;
        Ok(())
    }

}
