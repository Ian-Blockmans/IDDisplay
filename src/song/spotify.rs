
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use warp::{
    http::Response,
    Filter,
};
use std::collections::HashMap;
use rspotify::{self, scopes, AuthCodeSpotify, Config, Credentials, OAuth, Token};
use rspotify::model::{AdditionalType, PlayableItem};
use rspotify::prelude::{BaseClient, OAuthClient};
use std::fs::{self,File};
use std::io::Write;
use tokio;
use local_ip_address::local_ip;

use super::Song;
use super::super::TMP_DIR;

//spotify
const CLIENT_ID: &str = "72707970d9254ea1baf38fff45afed06";
const CLIENT_SECRET: &str = "73715f9ba05b40e6890e1f7aab9d20e7";
pub static SP_CACHE_PATH: &str = "./spotify_cache/spotify_token_cache.json";  //todo should not be needed
static SP_GET_DELAY: u64 = 3;

pub async fn spotify_wait_for_token(auth: AuthCodeSpotify) -> Result<Token, String>{
    for _i in 0..60 {
        let code = fs::read_to_string(TMP_DIR.to_string() + "code.txt");
        match &code {
            Ok(code_s) => {
                //auth.request_token(code_s).await;
                match auth.request_token(code_s).await {
                    Ok(_o) => {
                        match rspotify::Token::from_cache(SP_CACHE_PATH) {
                            Ok(token) => {
                                return Ok(token);
                            }
                            Err(_err) => {
                                return Err("bad token".to_string());
                            }
                        }
                    }
                    Err(e) => {
                        return Err(e.to_string());
                    }
                }
            }
            Err(_e) => {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
    Err("timeout".to_string())
}

pub async fn spotify_get_current(sp_auth: AuthCodeSpotify) -> Result<Song, String>{
    tokio::time::sleep(std::time::Duration::from_secs(SP_GET_DELAY)).await;
    //let token = Token::from_cache(SP_CACHE_PATH);
    let mut s = Song::default();
    let spotify = sp_auth; // there sould always be a token when we enter this function
    
    match spotify.current_playing(None,Some(vec![&AdditionalType::Track])).await {
        Ok(current) => {
            if current != None{
                match current.unwrap().item.unwrap() {
                    PlayableItem::Track( t) => {
                        s.track_name = t.name;
                        s.artist_name.clear();                                                                  //track name
                        let mut a_it = t.artists.iter().peekable();
                        while let Some(a) = a_it.next() { // while the next artist name is not none
                            if !a_it.peek().is_none(){
                                s.artist_name = s.artist_name.clone() + a.name.as_str(); // add , to end of name to list multiple artists 
                                s.artist_name = s.artist_name.clone() + ", " ;
                            } else {
                                s.artist_name = s.artist_name.clone() + a.name.as_str(); // add the last name without ,
                            }
                            s.art_url = t.album.images[0].url.to_string();
                        }
                        
                        //get image moved to main to not pull image when i already have image
                        //match get_image(&t.album.images[0].url, s.track_name.replace(" ", "_") + ".jpg").await { 
                        //    Ok(path ) => {
                        //        s.art = path;
                        //    }
                        //    Err(e) => {
                        //        s.track_name = e;
                        //    }
                        //};
                    }
                    PlayableItem::Episode(_e) => {
                        //sould never happen
                    }
                }
            }
            Ok(s)
        }
        Err(e) => {
            match spotify.refresh_token().await {
                Ok(_o) => {
                    return Err(e.to_string());
                }
                Err(e) => {
                    return Err(e.to_string());
                }
            }
        }
    }
}

pub async fn spotify_callback(_sp_auth: AuthCodeSpotify){ //maybe remove sp_auth
    //let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 40)), 80);
    let socket = SocketAddr::new( local_ip().unwrap(), 80);
    //println!("{:?}", socket.clone());
    let route = warp::path!("callback")
        .and(warp::query::<HashMap<String, String>>())
        .map(move |map: HashMap<String, String>| {
            let mut response: Vec<String> = Vec::new();
            for (key, value) in map.into_iter() {
                if key == "code"{
                    let file = File::create_new(TMP_DIR.to_string() + "code.txt");
                    match file.unwrap().write(value.as_bytes()) {
                        Ok(_o) => {

                        }
                        Err(_e) => {
                            //todo error handling for file write maybe write to log file
                        }
                    }
                    
                }
                response.push(format!("{}={}", key, value))
            }
            Response::builder().body(response.join(";"))
    });
    
    tokio::spawn(warp::serve(route).run(socket));
    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
}

pub async fn spotify_init() -> AuthCodeSpotify {
    //let ip = local_ip().unwrap();
    let token = Token::from_cache(SP_CACHE_PATH);
    match token {
        Ok(t)=>{
            let config = Config {
                token_cached: true,
                cache_path: (SP_CACHE_PATH).into(),
                ..Default::default()
            };
            let oauth = OAuth {
                scopes: scopes!(
                    "user-read-currently-playing",
                    "playlist-modify-private",
                    "playlist-modify-public"
                ),
                redirect_uri: "http://iddisplay.local/callback".to_string(),
                ..Default::default()
            };
            let creds = Credentials::new(CLIENT_ID, CLIENT_SECRET);
            AuthCodeSpotify::from_token_with_config(t, creds, oauth, config)
        }
        Err(_e)=>{
            let config = Config {
                token_cached: true,
                cache_path: (SP_CACHE_PATH).into(),
                ..Default::default()
            };
            let oauth = OAuth {
                scopes: scopes!(
                    "user-read-currently-playing",
                    "playlist-modify-private",
                    "playlist-modify-public"
                ),
                redirect_uri: "http://iddisplay.local/callback".to_owned(),
                ..Default::default()
            };
            let creds = Credentials::new(CLIENT_ID, CLIENT_SECRET);
            AuthCodeSpotify::with_config(creds, oauth, config)
        }
        
    }
}

pub async fn spotify_qr(auth: AuthCodeSpotify) -> String{
    let qr = rspotify::AuthCodeSpotify::get_authorize_url(&auth, false).unwrap();
    //qr_code::Data::new(rspotify::AuthCodeSpotify::get_authorize_url(&self.sp_auth, false).unwrap()).unwrap(); //create login url
    qr
}