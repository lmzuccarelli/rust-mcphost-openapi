use anyhow::Result;
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client as HttpClient;
use std::io::Write;
use std::io::stdout;

use crate::model::{CompletionRequest, DataResponse};

#[async_trait]
pub trait ChatClient: Send + Sync {
    //async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
    async fn complete(&self, request: CompletionRequest) -> Result<()>;
}

pub struct OpenAIClient {
    api_key: String,
    client: HttpClient,
    base_url: String,
}

impl OpenAIClient {
    pub fn new(api_key: String, url: Option<String>, proxy: Option<bool>) -> Self {
        let base_url = url.unwrap_or("http://192.168.1.221:8080/v1/chat/completions".to_string());
        let proxy = proxy.unwrap_or(false);
        let client = if proxy {
            HttpClient::new()
        } else {
            HttpClient::builder()
                .no_proxy()
                .build()
                .unwrap_or_else(|_| HttpClient::new())
        };

        Self {
            api_key,
            client,
            base_url,
        }
    }

    #[allow(unused)]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
}

macro_rules! print_flush {
    ( $($t:tt)* ) => {
        {
            let mut h = stdout();
            write!(h, $($t)* ).unwrap();
            h.flush().unwrap();
        }
    }
}

#[async_trait]
impl ChatClient for OpenAIClient {
    async fn complete(&self, request: CompletionRequest) -> Result<()> {
        let mut response = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?
            .bytes_stream();

        let mut buf = Vec::new();
        while let Some(item) = response.next().await {
            for byte in item? {
                if byte == b'\n' {
                    let resp_val = String::from_utf8(buf.clone());
                    match resp_val {
                        Ok(value) => {
                            if value.len() > 0 {
                                let cleaned = value.split("data: ").nth(1).unwrap();
                                let json_resp = serde_json::from_str::<DataResponse>(&cleaned);
                                match json_resp {
                                    Ok(val) => print_flush!("{}", val.content),
                                    Err(err) => println!("parsing json {}", err.to_string()),
                                }
                            }
                        }
                        Err(_) => println!("string invalid"),
                    }
                    buf.clear();
                } else {
                    buf.push(byte);
                }
            }
        }
        println!("");
        Ok(())
    }
}
