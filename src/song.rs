
pub mod spotify;
pub mod rcognize;

use super::TMP_DIR;

#[derive(Debug, Clone)]
pub struct Song{
    pub track_name: String,
    pub artist_name: String,
    pub art_path: String,
    pub art_url: String,
}

impl Default for Song {
    fn default() -> Self {Song::default()}
}

impl Song {
    fn default() -> Song {
        Song{ 
            track_name: "nosong".to_string(),
            artist_name: "Artistname".to_string(),
            art_path: "./unknown.png".to_string(),
            art_url: "".to_string(),
        }
    }
}

pub async fn get_image(link: String,store: String) -> Result<String, String> { //store = image name returns the path or error
    let target = &link;
    let response;
    match reqwest::get(target).await {
        Ok(r) => {
            match r.bytes().await {
                Ok(r) => {
                    response = r;
                }
                Err(e) => {
                    return Err(e.to_string());
                }
            }
        }
        Err(e) => {
            return Err(e.to_string());
        }        
    }

    match image::load_from_memory(&response) {
        Ok(img) => {
            match img.save(TMP_DIR.to_string() + &store) {
                Ok( _img) => {
                    return Ok(TMP_DIR.to_string() + &store);
                }
                Err(e) => {
                    return Err(e.to_string());
                }
            }
        }
        Err(e) => {
            return Err(e.to_string());
        }
    }
}