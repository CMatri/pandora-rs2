use super::crypt::decrypt;
use super::error::Result;
use super::method::Method;
use super::request::request;
use super::DEFAULT_ENDPOINT;
use reqwest::Client;
use serde_json;

#[derive(Debug)]
pub struct Credentials {
    username: String,
    password: String,
    encrypt_key: String,
    decrypt_key: String,
    partner_id: Option<String>,
    partner_auth_token: Option<String>,
    sync_time: Option<u64>,
    user_id: Option<String>,
    user_auth_token: Option<String>,
}

impl Credentials {
    pub fn new(username: &str, password: &str) -> Result<Self> {
        let client = Client::new();
        let partner = Partner::default();
        let mut credentials = Credentials {
            username: username.to_owned(),
            password: password.to_owned(),
            encrypt_key: partner.encrypt_password.clone(),
            decrypt_key: partner.decrypt_password.clone(),
            partner_id: None,
            partner_auth_token: None,
            sync_time: None,
            user_id: None,
            user_auth_token: None,
        };

        let partner_login: PartnerLogin = request(
            &client,
            DEFAULT_ENDPOINT,
            Method::AuthPartnerLogin,
            Some(serde_json::to_value(&partner)?),
            None,
        )?;
        credentials.set_partner_login(partner_login);

        let user_login_body = serde_json::to_value(&UserLoginRequest::new(
            username.to_owned(),
            password.to_owned(),
        ))
        .expect("Fatal error creating user login body");
        let user_login: UserLogin = request(
            &client,
            DEFAULT_ENDPOINT,
            Method::AuthUserLogin,
            Some(user_login_body),
            Some(&credentials),
        )
        .expect("Fatal error requesting user creds");
        credentials.set_user_login(user_login);

        Ok(credentials)
    }

    pub fn refresh(&mut self) -> Result<()> {
        match Credentials::new(&self.username, &self.password) {
            Ok(new_credentials) => {
                *self = new_credentials;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn encrypt_key(&self) -> &str {
        &self.encrypt_key
    }

    pub fn decrypt_key(&self) -> &str {
        &self.decrypt_key
    }

    pub fn partner_id<'a>(&'a self) -> Option<&'a str> {
        match self.partner_id {
            Some(ref partner_id) => Some(partner_id.as_str()),
            None => None,
        }
    }

    pub fn partner_auth_token<'a>(&'a self) -> Option<&'a str> {
        match self.partner_auth_token {
            Some(ref partner_auth_token) => Some(partner_auth_token.as_str()),
            None => None,
        }
    }

    pub fn sync_time<'a>(&'a self) -> Option<&'a u64> {
        match self.sync_time {
            Some(ref sync_time) => Some(&sync_time),
            None => None,
        }
    }

    pub fn user_id<'a>(&'a self) -> Option<&'a str> {
        match self.user_id {
            Some(ref user_id) => Some(user_id.as_str()),
            None => None,
        }
    }

    pub fn user_auth_token<'a>(&'a self) -> Option<&'a str> {
        match self.user_auth_token {
            Some(ref user_auth_token) => Some(user_auth_token.as_str()),
            None => None,
        }
    }

    fn set_partner_login(&mut self, partner_login: PartnerLogin) {
        use std::str;

        let sync_time_bytes: Vec<u8> = decrypt(self.decrypt_key(), &partner_login.sync_time)
            .iter()
            .skip(4)
            .cloned()
            .collect();
        let sync_time_str = str::from_utf8(&sync_time_bytes).unwrap_or("0");

        let sync_time = sync_time_str.parse::<u64>().unwrap_or(0);

        self.partner_id = Some(partner_login.partner_id.clone());
        self.partner_auth_token = Some(partner_login.partner_auth_token.clone());
        self.sync_time = Some(sync_time);
    }

    fn set_user_login(&mut self, user_login: UserLogin) {
        self.user_id = user_login.user_id.clone();
        self.user_auth_token = Some(user_login.user_auth_token.clone());
    }
}

#[derive(Serialize)]
pub struct Partner {
    username: String,
    password: String,
    #[serde(rename = "deviceModel")]
    device_model: String,
    version: String,
    #[serde(rename = "encryptPassword")]
    encrypt_password: String,
    #[serde(rename = "decryptPassword")]
    decrypt_password: String,
}

impl Default for Partner {
    fn default() -> Self {
        Partner {
            username: "android".to_owned(),
            password: "AC7IBG09A3DTSYM4R41UJWL07VLN8JI7".to_owned(),
            device_model: "android-generic".to_owned(),
            version: "5".to_owned(),
            encrypt_password: "6#26FRL$ZWD".to_owned(),
            decrypt_password: "R=U!LH$O2B#".to_owned(),
        }
    }
}

impl Partner {
    pub fn new(
        username: String,
        password: String,
        device_model: String,
        version: String,
        encrypt_password: String,
        decrypt_password: String,
    ) -> Self {
        Partner {
            username: username,
            password: password,
            device_model: device_model,
            version: version,
            encrypt_password: encrypt_password,
            decrypt_password: decrypt_password,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CheckLicensing {
    #[serde(rename = "isAllowed")]
    pub is_allowed: bool,
}

#[derive(Debug, Deserialize)]
pub struct PartnerLogin {
    #[serde(rename = "partnerId")]
    pub partner_id: String,
    #[serde(rename = "partnerAuthToken")]
    pub partner_auth_token: String,
    #[serde(rename = "syncTime")]
    pub sync_time: String,
}

#[derive(Debug, Deserialize)]
pub struct UserLogin {
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    #[serde(rename = "userAuthToken")]
    pub user_auth_token: String,
}

#[derive(Serialize)]
struct UserLoginRequest {
    #[serde(rename = "loginType")]
    login_type: String,
    username: String,
    password: String,
}

impl UserLoginRequest {
    pub fn new(username: String, password: String) -> Self {
        UserLoginRequest {
            login_type: "user".to_owned(),
            username: username,
            password: password,
        }
    }
}
