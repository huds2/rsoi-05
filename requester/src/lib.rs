use std::{collections::HashMap, error::Error};
use dyn_clone::{clone_trait_object, DynClone};
use async_trait::async_trait;
use custom_error::custom_error;
use serde::de::DeserializeOwned;

custom_error!{pub RequesterError
    ResponseCodeError                            = "Request did not return 2** code",
}


pub enum RequestMethod {
    GET,
    POST,
    DELETE
}

#[derive(Debug, Clone)]
pub struct Response {
    pub code: u16,
    pub body: String,
    pub header: HashMap<String, String>
}

#[async_trait]
// pub trait Requester<'a, T: Deserialize<'a>>: Sync + Send + DynClone {
pub trait Requester: Sync + Send + DynClone {
// pub trait Requester<'a, T: Deserialize<'a> + Clone>: Sync + Send + DynClone {
    async fn send(&mut self, url: String, method: RequestMethod, headers: HashMap<String, String>, body: String) -> Result<Response, Box<dyn Error>>;
}


clone_trait_object!(Requester);

#[derive(Clone)]
pub struct Reqwester {}

#[async_trait]
impl Requester for Reqwester {
    async fn send(&mut self, url: String, method: RequestMethod, headers: HashMap<String, String>, body: String) -> Result<Response, Box<dyn Error>> {
        let client = reqwest::Client::new();
        let mut builder = match method {
            RequestMethod::GET => client.get(url),
            RequestMethod::POST => client.post(url),
            RequestMethod::DELETE => client.delete(url)
        };
        builder = builder.body(body);
        for header in headers {
            builder = builder.header(header.0, header.1);
        }
        let response = builder.send().await?;
        let mut response_headers = HashMap::new();
        for header in response.headers() {
            response_headers.insert(header.0.to_string(), header.1.to_str()?.to_string());
        }
        Ok(Response {
            code: response.status().as_u16(),
            body: response.text().await?,
            header: response_headers
        })
    }
}

pub async fn send_typed<T: DeserializeOwned>(requester: &mut Box<dyn Requester>, url: String, method: RequestMethod, headers: HashMap<String, String>, body: String) -> Result<T, Box<dyn Error>> {
    let response = requester.send(url, method, headers, body).await?;
    if response.code == 200 || response.code == 201 || response.code == 204 {
        let value: T = serde_json::from_str::<T>(&response.body)?;
        return Ok(value);
    }
    Err(Box::new(RequesterError::ResponseCodeError))
}
