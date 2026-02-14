use std::{collections::HashMap, ops::Deref, sync::Arc, time::Duration};

use bucket::{BucketKey, Buckets};
use error::HttpError;
use reqwest::{
    Method, RequestBuilder,
    header::{HeaderMap, HeaderValue},
};
use serde::{Serialize, de::DeserializeOwned};
use volty_types::RevoltConfig;

mod bucket;
pub mod error;
pub mod routes;

pub use error::ApiError;

#[derive(Clone)]
pub struct Http {
    inner: Arc<InnerHttp>,
}

impl Deref for Http {
    type Target = InnerHttp;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct InnerHttp {
    api_url: String,

    buckets: Buckets,
    /// all requests will contain token
    pub client: reqwest::Client,
}

pub struct Request {
    bucket: BucketKey,
    request: RequestBuilder,
}

impl Request {
    pub fn json<J: Serialize>(mut self, json: &J) -> Self {
        self.request = self.request.json(json);
        self
    }
}

impl Http {
    pub fn new(token: impl std::fmt::Display, is_bot: bool) -> Self {
        const DEFAULT_API_URL: &str = "https://api.stoat.chat/";
        Self::with_api_url(DEFAULT_API_URL, token, is_bot)
    }

    pub fn with_api_url(
        api_url: impl std::fmt::Display,
        token: impl std::fmt::Display,
        is_bot: bool,
    ) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            if is_bot {
                "x-bot-token"
            } else {
                "x-session-token"
            },
            HeaderValue::from_str(&token.to_string()).unwrap(),
        );
        rustls::crypto::ring::default_provider()
            .install_default()
            .unwrap();
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();
        let inner = InnerHttp {
            api_url: api_url.to_string(),
            buckets: Buckets::new(),
            client,
        };
        Self {
            inner: Arc::new(inner),
        }
    }

    pub(crate) fn request(&self, method: Method, path: &str) -> Result<Request, HttpError> {
        let url = format!("{}/{}", self.api_url, path);
        let bucket = BucketKey::new(method.clone(), path);
        if let Err(e) = self.buckets.take(&bucket) {
            return Err(ApiError::RetryAfter(e).into());
        }
        let request = self.client.request(method, url);
        Ok(Request { bucket, request })
    }

    async fn send_request<T: DeserializeOwned>(&self, request: Request) -> Result<T, HttpError> {
        log::debug!("Request: {:?}", &request.request);
        let response = request.request.send().await;
        self.handle_response(response, request.bucket).await
    }

    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: Result<reqwest::Response, reqwest::Error>,
        bucket: BucketKey,
    ) -> Result<T, HttpError> {
        match response {
            Ok(response) => {
                self.buckets.handle_response(&bucket, &response);
                let is_success = response.status().is_success();
                let status_code = response.status().as_u16();
                let text = response.text().await?;

                if is_success {
                    let t = serde_json::from_str(&text)?;
                    Ok(t)
                } else if status_code == 429 {
                    let millis = serde_json::from_str::<HashMap<String, u64>>(&text)
                        .map(|m| *m.get("retry_after").unwrap_or(&10_000))
                        .unwrap_or(10_000);
                    Err(HttpError::Api(ApiError::RetryAfter(Duration::from_millis(
                        millis,
                    ))))
                } else {
                    log::error!("Failure status code: {status_code}, {text}");
                    let e = serde_json::from_str::<ApiError>(&text)?;
                    Err(HttpError::Api(e))
                }
            }
            Err(e) => {
                log::error!("Response: {:?}", e);
                Err(ApiError::LabelMe.into())
            }
        }
    }

    pub async fn api_info(&self) -> Result<RevoltConfig, HttpError> {
        let request = self.request(Method::GET, "")?;
        self.send_request(request).await
    }
}
