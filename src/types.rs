use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Tokens{
    pub access_token: String,
    pub refresh_token: String, 
    pub expires_in: i64
}

#[derive(Debug, PartialEq)]
pub enum Scope{
    Trading,
    Accounts
}