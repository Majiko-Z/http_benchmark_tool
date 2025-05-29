use clap::Parser;
use config::{Config,File};
use once_cell::sync::{Lazy, OnceCell};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::process::exit;
use ftlog::{info, error};
use anyhow::{Result, bail};
use tokio;
use reqwest::Client;
use std::time::Instant;

pub static CONFIG_FILE: OnceCell<String> = OnceCell::new();
pub static PROCESS_NUM: OnceCell<i64> = OnceCell::new();
pub static TOTAL_HTTP_NUM: OnceCell<i64> = OnceCell::new();
pub static QPS_NUM: OnceCell<i64> = OnceCell::new();

/// HTTP连接超时时间
pub static EXPIRE_TIME_SEC: OnceCell<i64> = OnceCell::new();

pub static HTTP_QUERY: Lazy<HttpQuery> =
    Lazy::new(||
        HttpQuery::new(CONFIG_FILE.get().unwrap().as_str()).unwrap_or_else(|e| {
            error!("Failed to load file:{}.", CONFIG_FILE.get().unwrap());
            exit(0)
        })
    );



#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct CMDs {
    #[arg(short = 'M', long = "mode")]
    pub mode: String, // process | qps
    #[arg(short = 'H', long = "http_file")]
    pub http_file: String,
    #[arg(short = 'P', long = "process_num")]
    pub process_num: i64,
    #[arg(short = 'T', long = "total")]
    pub total_num: i64,
    #[arg(short = 'Q', long = "qps")]
    pub qps: i64,
    #[arg(short = 'E', long = "expire_secs")]
    pub expire_secs: i64,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}
impl HttpRequest {
    pub async fn send_request(&self, client: Option<&Client>) -> Result<()> {
        let client = match client {
            Some(client) => client.clone(),
            None => Client::new(),
        };

        let mut request:Result<reqwest::RequestBuilder, reqwest::Error> = match self.method.as_str() {
            "POST" | "post" => Ok(client.post(&self.url).body(self.body.clone().into_bytes())),
            "GET" | "get" => Ok(client.get(&self.url)),
            _ => bail!("Invalid HTTP method: {}", self.method),
        };

        if request.is_err() {
            bail!("{}", request.err().unwrap());
        }
        let mut request = request.unwrap();
        for (k, v) in self.headers.iter() {
            request = request.header(k, v);
        }
        info!("Sending HTTP request. URL:{}, Method:{}, Headers:{:?}, Body:{}",
            self.url, self.method, self.headers, self.body);
        let start_time = Instant::now();
        let resp = request.send().await?;
        let duration = start_time.elapsed().as_millis();
        info!("Cost:{} ms, Response: {:?}", duration, resp);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpQuery {
    pub query: Vec<HttpRequest>,
}

impl HttpQuery {
    pub fn new(path: &str) -> Result<Self> {
        let f_content = fs::read_to_string(path)?;
        let query =  match serde_json::from_str(&f_content) {
            Ok(q) => {q},
            Err(e) => { // 这里可以续借其他格式
                return bail!("{}", e.to_string());
            }
        };
        Ok(query)
    }
    pub fn pick_config(&self) -> Self {
        self.clone()
    }

}
