use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use warp::{
    http::Response,
    Filter,
};
use std::collections::HashMap;
use rspotify::{self, scopes, AuthCodeSpotify, Config, Credentials, OAuth, Token};
use rspotify::model::{AdditionalType, PlayableItem};
use rspotify::prelude::OAuthClient;
use std::fs::{self,File};
use std::io::Write;

use super::Song;
use super::SongOrigin;

//spotify
//const CACHE_PATH: &str = "./tmp/";
const CLIENT_ID: &str = "72707970d9254ea1baf38fff45afed06";
const CLIENT_SECRET: &str = "73715f9ba05b40e6890e1f7aab9d20e7";

pub static TMP_DIR_S: &str = "./tmp/";  //todo should not be needed

pub async fn spotify_wait_for_token(auth: AuthCodeSpotify) -> Result<Token, String>{
    let tmp_path = TMP_DIR_S;
    //let code = fs::read_to_string(tmp_path.to_string() + "code.txt");
    for _i in 0..60 {
        let code = fs::read_to_string(tmp_path.to_string() + "code.txt");
        match &code {
            Ok(code_s) => {
                //auth.request_token(code_s).await;
                match auth.request_token(code_s).await {
                    Ok(_o) => {
                        match rspotify::Token::from_cache(TMP_DIR_S.to_string() + "spotify_token_cache.json") {
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
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
    Err("timeout".to_string())
}

pub async fn spotify_get_current(token: Token) -> Song{
    let mut s = Song::default();
    let spotify = AuthCodeSpotify::from_token(token);
    let current = spotify.current_playing(None,Some(vec![&AdditionalType::Track])).await.unwrap();
    if current != None{
        match current.unwrap().item.unwrap() {
            PlayableItem::Track( t) => {
                s.track_name = t.name;
                s.origin = SongOrigin::Spotify
            }
            PlayableItem::Episode(_e) => {
                //sould never happen
            }
        }
    }
    s
}

pub async fn spotify_callback(_sp_auth: AuthCodeSpotify) -> String { //maybe remove sp_auth
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 40)), 80);
    let route = warp::path!("callback")
        .and(warp::query::<HashMap<String, String>>())
        .map(move |map: HashMap<String, String>| {
            let mut response: Vec<String> = Vec::new();
            for (key, value) in map.into_iter() {
                if key == "code"{
                    let tmp_dir = TMP_DIR_S;
                    let file = File::create_new(tmp_dir.to_string() + "code.txt");
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
    
    warp::serve(route)
        .run(socket).await;
    "Ok".to_string()
}

pub async fn spotify_init() -> AuthCodeSpotify {
    let config = Config {
        token_cached: true,
        cache_path: (TMP_DIR_S.to_string() + "spotify_token_cache.json").into(),
        ..Default::default()
    };
    let oauth = OAuth {
        scopes: scopes!(
            "user-read-currently-playing",
            "playlist-modify-private",
            "playlist-modify-public"
        ),
        redirect_uri: "http://desktop.local/callback".to_owned(),
        ..Default::default()
    };
    let creds = Credentials::new(CLIENT_ID, CLIENT_SECRET);
    AuthCodeSpotify::with_config(creds, oauth, config)
}

pub async fn spotify_qr(auth: AuthCodeSpotify) -> String{
    let qr = rspotify::AuthCodeSpotify::get_authorize_url(&auth, false).unwrap();
    //qr_code::Data::new(rspotify::AuthCodeSpotify::get_authorize_url(&self.sp_auth, false).unwrap()).unwrap(); //create login url
    qr
}