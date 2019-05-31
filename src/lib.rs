// This mod is almost completely copied from https://github.com/danielrs/pandora-rs/. I've simply replaced the use of hyper with reqwest and updated syntax slightly. 
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use reqwest::Client;

pub mod method;
pub mod crypt;
pub mod auth;
pub mod request;
pub mod response;
pub mod error;
pub mod stations;
pub mod playlist;
pub mod music;

pub use auth::Credentials;
pub use playlist::Track;
pub use stations::Stations;

use serde::de::DeserializeOwned;
use serde_json::value::Value;

use request::request;
use method::Method;
use error::{Error, Result};

use std::sync::Mutex;
use std::cell::RefCell;

#[derive(Debug)]
pub struct Pandora {
    client: Client,
    endpoint: Endpoint<'static>,
    credentials: Mutex<RefCell<Credentials>>
}

impl Pandora {
    pub fn new(username: &str, password: &str) -> Result<Self> {
        let creds = Credentials::new(username, password)?;
        Ok(Pandora::with_credentials(creds))
    }

    pub fn with_credentials(credentials: Credentials) -> Self {
        Pandora {
            client: Client::new(),
            endpoint: DEFAULT_ENDPOINT,
            credentials: Mutex::new(RefCell::new(credentials))
        }
    }

    pub fn stations(&self) -> Stations {
        Stations::new(self)
    }

    pub fn request<T>(&self, method: Method, body: Option<Value>) -> Result<T> where T: DeserializeOwned {
        let credentials = self.credentials.lock().unwrap();
        let req = request(&self.client, self.endpoint, method.clone(), body.clone(), Some(&credentials.borrow()));

        match req {
            Ok(res) => Ok(res),
            Err(err) => {
                if credentials.borrow_mut().refresh().is_err() { return Err(err); }
                request(&self.client, self.endpoint, method, body, Some(&credentials.borrow()))
            }
        }
    }

    pub fn request_noop(&self, method: Method, body: Option<Value>) -> Result<()> {
        let credentials = self.credentials.lock().unwrap();

        let req = request::<()>(&self.client, self.endpoint, method.clone(), body.clone(), Some(&credentials.borrow()));

        match req {
            Ok(_) |  Err(Error::Codec(_)) => Ok(()),
            Err(err) => {
                if credentials.borrow_mut().refresh().is_err() { return Err(err); }
                let req = request::<()>(&self.client, self.endpoint, method, body, Some(&credentials.borrow()));
                match req {
                    Ok(_) | Err(Error::Codec(_)) => Ok(()),
                    Err(err) => Err(err),
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Endpoint<'a>(&'a str);

impl<'a> ToString for Endpoint<'a> {
    fn to_string(&self) -> String {
        let Endpoint(url) = *self;
        url.to_owned()
    }
}

pub const ENDPOINTS: [Endpoint<'static>; 4] =
    [Endpoint("http://tuner.pandora.com/services/json/"),
     Endpoint("https://tuner.pandora.com/services/json/"),
     Endpoint("http://internal-tuner.pandora.com/services/json/"),
     Endpoint("https://internal-tuner.pandora.com/services/json/")];
pub const DEFAULT_ENDPOINT: Endpoint<'static> = ENDPOINTS[1];