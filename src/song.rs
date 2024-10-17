
pub mod spotify;
pub mod rcognize;

#[derive(Debug, Clone)]
pub enum SongOrigin{
    Shazam,
    Spotify,
}

#[derive(Debug, Clone)]
pub struct Song{
    pub track_name: String,
    pub artist_name: String,
    pub art: String,
    pub error: String,
    pub origin: SongOrigin,
}

impl Default for Song {
    fn default() -> Self {Song::default()}
}

impl Song {
    fn default() -> Song {
        Song{ 
            track_name: "nosong".to_string(),
            artist_name: "Artistname".to_string(),
            art: "./unknown.png".to_string(),
            error: "Ok".to_string(),
            origin: SongOrigin::Spotify,
        }
    }
}