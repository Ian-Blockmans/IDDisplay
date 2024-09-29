use core::str;
use std::process::Command;
use std::result::Result;
use std::str::Utf8Error;
use std::string;
use serde_json::Result as JResult;
use serde_json::Value;

struct Song{
    track_name: String,
    artist_name: String,
    art: String,
}

//impl Song {
//    fn update(&mut self, message: Message) {
//        match message {
//            Message::Detect => {}
//            Message::Exit => {}
//        }
//    }
//}

//enum Message {
//    Detect,
//    Exit,
//}

#[derive(Debug)]
enum MainError {
    CmdErr(std::io::Error),
    PyErr(String),
    JsonErr,
    ParseErr(Utf8Error),
}

impl MainError {
    
    //fn from_cmderr(err: std::io::Error) -> Self{
    //    Self::CmdErr(err)
    //} 
}

fn shazamrec() -> Result<String, MainError> {
    let output = Command::new("python")
        .args(["ShazamIO.py", "MPH-CrowdRolling.ogg"])
        .output().map_err(MainError::CmdErr)?;
    let pyerrout = str::from_utf8(&output.stderr).unwrap();
    if pyerrout.is_empty(){
        Ok(str::from_utf8(&output.stdout).map_err(MainError::ParseErr)?.to_string())
    } else{
        let errorout = str::from_utf8(&output.stderr).map_err(MainError::ParseErr)?;
        println!("Error: {}", errorout);
        Err(MainError::PyErr(errorout.to_string()))
    }
   
}

fn main() -> Result<(), MainError> {
    let jstring = shazamrec()?;
    println!("song: {}", jstring);
    let shazam_json_p: Value = serde_json::from_str(&jstring).unwrap();
    //shazam_json_p.unwrap().as_str()
    //let shazam_data = shazam_json_p["track"]["title"];
    //let track: String = shazam_json_p["track"]["title"].into();
    let song1 = Song { 
        track_name: shazam_json_p["track"]["title"].as_str().unwrap().to_string(), 
        artist_name: shazam_json_p["track"]["title"].as_str().unwrap().to_string(), 
        art: shazam_json_p["track"]["images"]["coverart"].as_str().unwrap().to_string(), 
    };
    println!("song: {}", song1.track_name);
    Ok(())
}