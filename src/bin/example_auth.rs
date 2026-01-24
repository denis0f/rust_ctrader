use rust_ctrader::{AuthClient, Scope};
use std::{env, io::stdin};
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    dotenv().ok();

    let client_id = env::var("client_id")?;
    let client_secret = env::var("client_secret")?;
    let redirect_uri = env::var("redirect_uri")?;

    let client = AuthClient::new(&client_id, &client_secret, &redirect_uri);
    let scope = Scope::Trading;

    let mut code_buf = String::new();

    //getting the authorization url to visit so as to get the code which will be used in gettting access tokens 

    let authorization_url = client.get_authorization_url(scope);

    println!("visit the url below to authorize the accounts and get the authorization code:\n {}", authorization_url);

    println!("Enter the code you have gotten from the redirected page:");
    stdin()
    .read_line(&mut code_buf)?;

    println!("\n====================\n");
    println!("The code recieved is: {}", code_buf.trim());

    //now lets get the access tokens

    let tokens = client.get_access_tokens(&code_buf.trim()).await?;

    println!("Got the access tokens: {:#?}", tokens);
    //after printing the access tokens you can now copy and paste the access tokens that is the access_token and the refresh token in the .env file or just export them in your environment 

    //now lets try to refresh our token (this is done after you have set the refresh token in the .env file or your environment variables )
    let refresh_token = env::var("refresh_token")?;

    let refreshed_tokens = client.refresh_token(&refresh_token).await?;

    println!("the refreshed tokens are: {:#?}", refreshed_tokens);

    Ok(())
}