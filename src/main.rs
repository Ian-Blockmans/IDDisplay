use core::str;
use std::{fmt::Error, process::{Command, Output}};
use serde_json::{Result, Value};

//struct Song{
//    track_name: String,
//    artist_name: String,
//    art: String,
//}
//enum Message {
//    Detect,
//    Exit,
//}
//
//impl Song {
//    fn update(&mut self, message: Message) {
//        match message {
//            Message::Detect => {}
//            Message::Exit => {}
//        }
//    }
//}

//enum MainError {
//    PyError,
//}

fn shazamrec() -> String {
    let output = Command::new("python")
        .args(["ShazamIO.py", "MPH-CrowdRolling.ogg"])
        .output().unwrap();
    println!("{}", str::from_utf8(&output.stderr).unwrap());
    str::from_utf8(&output.stdout).unwrap().to_owned()
}

fn main() {
    let shazam_json_p: Value = serde_json::from_str(shazamrec().as_str()).unwrap();
    //shazam_json_p.unwrap().as_str()
    //let shazam_data = shazam_json_p["track"]["title"];
    //let track: String = shazam_json_p["track"]["title"].into();
    println!("{}", shazam_json_p["track"]["title"]);

}