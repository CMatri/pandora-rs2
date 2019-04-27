use serde::de::DeserializeOwned;
use serde_json;
use serde_json::value::{ Number, Value };

use url::Url;

use reqwest::{ Body, Client };

use super::crypt;
use super::method::Method;
use super::Endpoint;
use super::Credentials;
use super::response::{ Stat, Response };
use super::error::{ Error, Result };

pub fn request<T>(client: &Client, endpoint: Endpoint, method: Method, body: Option<Value>, credentials: Option<&Credentials>) -> Result<T> where T: DeserializeOwned {
    let mut body = serde_json::to_string(&build_body(body, credentials))?;
    
    if method.is_encrypted() {
        if let Some(credentials) = credentials {
            body = crypt::encrypt(credentials.encrypt_key(), &body);
        }
    }


    let url = build_url(client, endpoint, method, credentials);
    let mut res_post = client.post(&url.to_string())
        .body(Body::from(body))
        .send()?;
    let res: Response<T> = res_post.json()?;
    
    match res {
        Response {
            stat: Stat::Ok,
            result: Some(result),
            ..
        } => Ok(result),
        Response {
            stat: Stat::Ok,
            result: None,
            ..
        } => Err(Error::Io(<std::io::Error>::new(std::io::ErrorKind::NotFound, "Result not found."))),
        Response { stat: Stat::Fail, .. } => {
            Err(Error::Api {
                message: res.message.unwrap(),
                code: res.code.unwrap().into()
            })
        }
    }
}

fn build_body(body: Option<Value>, credentials: Option<&Credentials>) -> Value {
    let mut body = match body {
        Some(body) => body,
        None => serde_json::to_value(serde_json::Map::<String, Value>::new()).expect("Fatal error building body.")
    };
    if let Some(credentials) = credentials {
        if let Some(obj) = body.as_object_mut() {
            if let Some(partner_auth_token) = credentials.partner_auth_token() {
                obj.insert("partnerAuthToken".to_owned(),
                           Value::String(partner_auth_token.to_owned()));
            }
            if let Some(sync_time) = credentials.sync_time() {
                obj.insert("syncTime".to_owned(), Value::from(*sync_time));
            }
            if let Some(user_auth_token) = credentials.user_auth_token() {
                obj.insert("userAuthToken".to_owned(),
                           Value::String(user_auth_token.to_owned()));
            }
        }
    }
    body
}

fn build_url(client: &Client, endpoint: Endpoint, method: Method, credentials: Option<&Credentials>) -> Url {
    let url = format!("{}?method={}", endpoint.to_string(), method.to_string());
    let mut url = Url::parse(&url).unwrap();
    if let Some(credentials) = credentials {
        use std::collections::BTreeMap;
        let mut query_pairs: BTreeMap<&str, &str> = BTreeMap::new();
        if let Some(partner_auth_token) = credentials.partner_auth_token() {
            query_pairs.insert("auth_token", partner_auth_token);
        }
        if let Some(user_auth_token) = credentials.user_auth_token() {
            query_pairs.insert("auth_token", user_auth_token);
        }
        if let Some(partner_id) = credentials.partner_id() {
            query_pairs.insert("partner_id", partner_id);
        }
        if let Some(user_id) = credentials.user_id() {
            query_pairs.insert("user_id", user_id);
        }
        url.query_pairs_mut().extend_pairs(query_pairs);
    }
    url
}