use crate::domain::errors::DomainError;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Clone)]
pub struct SupabaseClient {
    base_url: String,
    api_key: String,
    http: Client,
}

impl SupabaseClient {
    pub fn new(url: String, api_key: String) -> Result<Self, DomainError> {
        let mut headers = HeaderMap::new();
        let key_val = HeaderValue::from_str(&api_key)
            .map_err(|e| DomainError::Infrastructure(format!("Invalid Supabase API key: {}", e)))?;
        let auth_val = HeaderValue::from_str(&format!("Bearer {}", api_key))
            .map_err(|e| DomainError::Infrastructure(format!("Invalid Supabase Auth header: {}", e)))?;

        headers.insert("apikey", key_val);
        headers.insert(AUTHORIZATION, auth_val);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let http = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| DomainError::Infrastructure(format!("Failed to build HTTP client: {}", e)))?;

        let clean_url = url.trim_end_matches('/').to_string();

        Ok(Self {
            base_url: clean_url,
            api_key,
            http,
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn from_env() -> Option<Self> {
        let url = std::env::var("SUPABASE_URL").ok()?;
        let key = std::env::var("SUPABASE_SERVICE_ROLE_KEY")
            .or_else(|_| std::env::var("SUPABASE_KEY"))
            .ok()?;
        Self::new(url, key).ok()
    }

    pub fn rest_url(&self, table: &str) -> String {
        format!("{}/rest/v1/{}", self.base_url, table)
    }

    pub async fn select<T: DeserializeOwned>(
        &self,
        table: &str,
        query_params: &[(&str, &str)],
    ) -> Result<Vec<T>, DomainError> {
        let url = self.rest_url(table);
        let res = self
            .http
            .get(&url)
            .query(query_params)
            .send()
            .await
            .map_err(|e| DomainError::Infrastructure(format!("Supabase GET error: {}", e)))?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(DomainError::Infrastructure(format!(
                "Supabase SELECT failed ({}): {}",
                status, body
            )));
        }

        res.json::<Vec<T>>()
            .await
            .map_err(|e| DomainError::Infrastructure(format!("Supabase JSON parse error: {}", e)))
    }

    pub async fn select_one<T: DeserializeOwned>(
        &self,
        table: &str,
        query_params: &[(&str, &str)],
    ) -> Result<Option<T>, DomainError> {
        let list: Vec<T> = self.select(table, query_params).await?;
        Ok(list.into_iter().next())
    }

    pub async fn insert<T: Serialize, R: DeserializeOwned>(
        &self,
        table: &str,
        record: &T,
    ) -> Result<Option<R>, DomainError> {
        let url = self.rest_url(table);
        let res = self
            .http
            .post(&url)
            .header("Prefer", "return=representation")
            .json(record)
            .send()
            .await
            .map_err(|e| DomainError::Infrastructure(format!("Supabase POST error: {}", e)))?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(DomainError::Infrastructure(format!(
                "Supabase INSERT failed ({}): {}",
                status, body
            )));
        }

        let results: Vec<R> = res
            .json()
            .await
            .map_err(|e| DomainError::Infrastructure(format!("Supabase JSON parse error: {}", e)))?;

        Ok(results.into_iter().next())
    }

    pub async fn upsert<T: Serialize>(
        &self,
        table: &str,
        record: &T,
        on_conflict: &str,
    ) -> Result<(), DomainError> {
        let url = self.rest_url(table);
        let res = self
            .http
            .post(&url)
            .header("Prefer", "resolution=merge-duplicates")
            .query(&[("on_conflict", on_conflict)])
            .json(record)
            .send()
            .await
            .map_err(|e| DomainError::Infrastructure(format!("Supabase UPSERT error: {}", e)))?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(DomainError::Infrastructure(format!(
                "Supabase UPSERT failed ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    pub async fn update<T: Serialize>(
        &self,
        table: &str,
        query_params: &[(&str, &str)],
        patch: &T,
    ) -> Result<(), DomainError> {
        let url = self.rest_url(table);
        let res = self
            .http
            .patch(&url)
            .query(query_params)
            .json(patch)
            .send()
            .await
            .map_err(|e| DomainError::Infrastructure(format!("Supabase PATCH error: {}", e)))?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(DomainError::Infrastructure(format!(
                "Supabase UPDATE failed ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    pub async fn delete(&self, table: &str, query_params: &[(&str, &str)]) -> Result<(), DomainError> {
        let url = self.rest_url(table);
        let res = self
            .http
            .delete(&url)
            .query(query_params)
            .send()
            .await
            .map_err(|e| DomainError::Infrastructure(format!("Supabase DELETE error: {}", e)))?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(DomainError::Infrastructure(format!(
                "Supabase DELETE failed ({}): {}",
                status, body
            )));
        }

        Ok(())
    }
}
