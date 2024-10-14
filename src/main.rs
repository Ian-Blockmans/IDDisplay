use core::{arch, error, str};
use std::borrow::Borrow;
use std::process::Command;
use std::{clone, env};
use clap::builder::Str;
use futures::{task, TryFutureExt};
use iced::border::{color, right};
use iced::widget::canvas::path::lyon_path::geom::euclid::default;
use iced::{color, run, settings, window, Background, Border, Color, Padding, Shadow, Size, Subscription, Task, Theme};
use iced::Element;
use iced::time::{self, Duration, Instant};
use rspotify::model::{AdditionalType, PlayableItem};
use rspotify::prelude::{BaseClient, OAuthClient};
use serde_json::Value;
use iced::{widget::{button, column, text, row, Column, Row, container, overlay, qr_code}, Length, Settings, font, Font, Alignment};
use iced::widget::{Button, Image as IceImage, QRCode};
use iced::widget::image as iceimage;
use hound;
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
//use tempfile::{tempdir, TempDir};
use std::fs::{self, remove_file, remove_dir_all, create_dir, File};
use std::io::{BufWriter, Read, Write};
use std::sync::{Arc, Mutex};
use anyhow::{Error, Result};
use std::collections::HashMap;
use warp::{
    http::Response,
    Filter,
};
use rspotify::{self, scopes, AuthCodeSpotify, Config, Credentials, OAuth, Token};
use tokio;
use std::net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr};

pub static CUSTOM_FONT: Font = Font::with_name("Less Perfect DOS VGA");
pub mod bytes {
    pub static CUSTOM_FONT: &[u8] = include_bytes!("../LessPerfectDOSVGA.ttf");
}

//spotify
//const CACHE_PATH: &str = "./tmp/";
const CLIENT_ID: &str = "72707970d9254ea1baf38fff45afed06";
const CLIENT_SECRET: &str = "73715f9ba05b40e6890e1f7aab9d20e7";

static TMP_DIR_S: &str = "./tmp/"; 
static REC_TIME_S: u64 = 3;
static EVERY_S: u64 = 3600; //run tick every amount of seconds
static WAIT_WHEN_CORRECT: u64 = 5;
static WAIT_REC: u64 = 1; //wait to slow down recognition, i don't want to spam shazam, might not be necessairy 
static OS: &str = env::consts::OS;
static ARCHITECTURE: &str = env::consts::ARCH;
static TEXT_SIZE: u16 = 60;


fn main() -> Result<(), anyhow::Error> {
    println!("OS: {}", OS);
    println!("Architecture: {}", ARCHITECTURE);
    let path = env::current_dir()?;
    println!("The current directory is {}", path.display());
    if fs::exists(TMP_DIR_S)? {
        remove_dir_all(TMP_DIR_S)?;
    }
    create_dir(TMP_DIR_S)?;
    let fontbytes = bytes::CUSTOM_FONT;
    let set = settings::Settings{
        id: Some("app".to_string()),
        fonts: vec![fontbytes.into()],
        default_font: CUSTOM_FONT,
        default_text_size: iced::Pixels(25.0),
        antialiasing: false,
    };
    //iced::application("ShazamDisplay", Song::update, Song::view).subscription(Song::songsubscription).run()?; // run songsubscription EVERY_S as a tick
    iced::application("ShazamDisplay", App::update, App::view)
        .settings(set)
        .theme(App::termtheme)
        .window_size(Size::new(800.0, 480.0))
        .run_with(App::startup)?;
    //iced::run("start", Song::update, Song::view)?;
    remove_dir_all(TMP_DIR_S)?;
    Ok(())
}

#[derive(Debug, Clone)]
struct Song{
    track_name: String,
    artist_name: String,
    art: String,
    error: String,
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
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Detect,
    Exit,
    DisplaySong(Song),
    Menu,
    SpInit,
    SpSaveAuth(AuthCodeSpotify),
    SpShowQr(String),
    SpAuthOk,
    SpAuthError(Result<Token, String>),
    SpShowCurrent,
    Demo,
    GetMainWinId,
    StoreMainWinId(window::Id),
    FullscreenExec(window::Id),
    Fullscreen,
    Tick,
    None(()),
}

#[derive(Debug)]
struct App{
    track_name: String,
    track_name_prev: [String; 5],
    prev_index: usize,
    artist_name: String,
    art: String,
    tmp_dir: String,
    winid: window::Id,
    correct: bool,
    sp_auth: rspotify::AuthCodeSpotify,
    sp_auth_url_data: qr_code::Data,
}

impl Default for App {
    fn default() -> Self { App::default()}
}

impl App {
    fn default() -> App {
        App{ 
            track_name: "nosong".to_string(),
            track_name_prev: ("Previous Track-name0".to_string(), "Previous Track-name1".to_string(), "Previous Track-name2".to_string(), "Previous Track-name3".to_string(), "Previous Track-name4".to_string()).into(),
            prev_index: 0,
            artist_name: "Artistname".to_string(),
            art: "./unknown.png".to_string(),
            tmp_dir: TMP_DIR_S.to_string(),
            winid: window::Id::unique(),
            correct: false,
            sp_auth: AuthCodeSpotify::default(),
            sp_auth_url_data: qr_code::Data::new( "http://localhost/").unwrap(),
        }
    }

    fn startup() -> (App, Task<Message>) {
        (App::default(), Task::none())
    }
    fn termtheme(&self) -> Theme {
        let terminal: iced::theme::Palette = iced::theme::Palette{
            background: Color{r: 0.0, g: 0.0, b:0.0, a: 1.0},
            text: Color{r: 0.12, g: 0.84, b:0.38, a: 1.0},
            primary: Color{r: 0.1, g: 0.1, b:0.1, a: 1.0},
            success: Color{r: 0.0, g: 0.0, b: 0.0, a: 1.0},
            danger: Color{r: 0.0, g: 0.0, b: 0.0, a: 1.0},
        };
        Theme::custom("Terminal".to_string(), terminal)
    }
    
    fn btntheme(theme: &Theme, status: button::Status) -> button::Style {
        let textcolor = theme.palette().text;
        match status {
            button::Status::Active => {
                let style = button::Style {
                    background: Some(Background::Color(Color{r: 0.0, g: 0.0, b:0.0, a: 1.0})),
                    text_color: textcolor,
                    border: Border::default(),
                    shadow: Shadow::default(),
                };
                style
            }
            _ => {
                let style = button::Style {
                    background: Some(Background::Color(Color{r: 0.1, g: 0.1, b:0.1, a: 1.0})),
                    text_color: textcolor,
                    border: Border::default(),
                    shadow: Shadow::default(),
                };
                style
            }
            //button::primary(theme, status),
        }
    }
    
    fn songsubscription(&self) -> Subscription<Message>{
        time::every(Duration::from_secs(EVERY_S)).map(|_| Message::Tick)
    }
    
    fn update(&mut self, message: Message) -> Task<Message>{
        self.tmp_dir = TMP_DIR_S.to_string();
        match message {
            Message::Tick => {
                iced::Task::perform(startrecasy(self.correct.clone(), self.tmp_dir.clone()), Message::DisplaySong)
            }
            Message::Detect => {
                iced::Task::perform(startrecasy(self.correct.clone(), self.tmp_dir.clone()), Message::DisplaySong)
            }
            Message::Exit => {
                iced::exit::<Message>()
            },
            Message::DisplaySong(song) => {
                if song.error != "Ok".to_string() {
                    self.track_name = "ShazamIO failed to execute".to_string();
                    Task::none()
                } else {
                    self.correct = false;
                    let mut matched = 0; //amount of time current song is found in previous
                    self.track_name_prev.iter().for_each(|s| if *s == song.track_name { matched += 1 }); //inc matched if a trackname matches
                    if matched >= 1{
                        self.correct = true; //set correct to true so the next thread will wait a bit before resuming scanning
                        self.track_name_prev = App::default().track_name_prev; //reset the previous song array
                    }

                    if song.track_name != "nosong".to_string(){
                        self.track_name_prev[self.prev_index] = song.track_name.clone(); //load current song in previous songs if not "nosong"
                    }
                    if self.prev_index >= 5 {self.prev_index = 0}

                    if song.track_name != "nosong" {
                        self.track_name = song.track_name;
                        self.artist_name = song.artist_name;
                        if self.track_name == "No song detected".to_string(){
                            self.art = "".to_string();
                        } else {
                            self.art = song.art.clone();
                        }
                        self.view();
                    }
                    iced::Task::perform(startrecasy(self.correct.clone(), self.tmp_dir.clone()), Message::DisplaySong)
                }
                
                //Task::none()
            },
            Message::Menu => {
                Task::done(Message::SpInit)
            },
            Message::Demo => {
                self.track_name = "Track Name".to_string();
                self.artist_name = "Artist Name".to_string();
                self.art = "./unknown.png".to_string();
                Task::none()
            },
            Message::FullscreenExec(id) => {
                iced::window::change_mode(id, window::Mode::Fullscreen) //change window id to fullschreen
            },
            Message::Fullscreen => {
                Task::done(Message::FullscreenExec(self.winid.clone()))
            },
            Message::GetMainWinId => {
                window::get_oldest().map(|id| Message::StoreMainWinId(id.unwrap())) //get oldest id and pass to FullscreenExec
            },
            Message::StoreMainWinId(id) => {
                self.winid = id;
                Task::none()
            }
            Message::None(n) => {
                Task::none()
            },
            Message::SpShowQr(url) => {
                self.sp_auth_url_data = qr_code::Data::new(url).unwrap();
                tokio::spawn(spotify_callback(self.sp_auth.clone()));
                Task::done(Message::SpAuthOk)
            },
            Message::SpAuthOk => {
                Task::perform(spotify_wait_for_token(self.sp_auth.clone()),Message::SpAuthError)
            },
            Message::SpAuthError(res) => {
                match res {
                    Ok(token) => {
                        Task::perform(spotify_get_current(token),Message::DisplaySong)
                    }
                    Err(error) =>{
                        self.track_name = error;
                        Task::none()
                    }
                }
            },
            Message::SpInit => {
                Task::perform(spotify_init(), Message::SpSaveAuth)
            },
            Message::SpSaveAuth(auth) => {
                self.sp_auth = auth;
                Task::perform(spotify_qr(self.sp_auth.clone()), Message::SpShowQr)
            },
            Message::SpShowCurrent => {
                Task::none()
            },
        }
    }

    fn view(&self) -> Element<Message>{
        let detect = button("detect")
            .on_press(Message::Detect)
            .style(Self::btntheme);
        let demo = button("demo")
            .on_press(Message::Demo)
            .style(Self::btntheme);
        let exit = button("exit")
            .on_press(Message::Exit)
            .style(Self::btntheme);
        let menu = button("menu")
            .on_press(Message::Menu)
            .style(Self::btntheme);
        let fullscreen = button("Fullscreen")
            .on_press(Message::Fullscreen)
            .style(Self::btntheme);
        
        let trackname = text(self.track_name.clone())
            .size(TEXT_SIZE)
            .center();
        let artistname= text(self.artist_name.clone())
            .size(TEXT_SIZE - 15)
            .center();

        
        let coverart = iceimage(self.art.clone())
            .width(300);
        
        let spotify_qr_code = qr_code(&self.sp_auth_url_data);

        container(
            column![
            row![ column![ row![ detect,exit,demo,fullscreen ] ].padding(5).width(Length::FillPortion(2)),column![ menu ].padding(5).align_x(Alignment::End).width(Length::FillPortion(1))],
            row![ column![ trackname, artistname ].padding(40).width(Length::FillPortion(6)).align_x(Alignment::Start), column![coverart].align_x(Alignment::End).width(Length::FillPortion(4)),],
            row![ spotify_qr_code ],
        ]).into()
    }
    
}

async fn spotify_wait_for_token(auth: AuthCodeSpotify) -> Result<Token, String>{
    let tmp_path = TMP_DIR_S;
    //let code = fs::read_to_string(tmp_path.to_string() + "code.txt");
    for i in 0..60 {
        let code = fs::read_to_string(tmp_path.to_string() + "code.txt");
        match &code {
            Ok(code_s) => {
                //auth.request_token(code_s).await;
                match auth.request_token(code_s).await {
                    Ok(o) => {
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

async fn spotify_get_current(token: Token) -> Song{
    let mut s = Song::default();
    let spotify = AuthCodeSpotify::from_token(token);
    let current = spotify.current_playing(None,Some(vec![&AdditionalType::Track])).await.unwrap();
    if current != None{
        match current.unwrap().item.unwrap() {
            PlayableItem::Track( t) => {
                s.track_name = t.name
            }
            PlayableItem::Episode(e) => {
                
            }
        }
    }
    s
}

async fn spotify_callback(sp_auth: AuthCodeSpotify) -> String {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 40)), 80);
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let route = warp::path!("callback")
        .and(warp::query::<HashMap<String, String>>())
        .map(move |map: HashMap<String, String>| {
            let mut response: Vec<String> = Vec::new();
            for (key, value) in map.into_iter() {
                if key == "code"{
                    let tmp_dir = TMP_DIR_S;
                    let file = File::create_new(tmp_dir.to_string() + "code.txt");
                    file.unwrap().write(value.as_bytes());
                }
                response.push(format!("{}={}", key, value))
            }
            Response::builder().body(response.join(";"))
    });
    
    warp::serve(route)
        .run(socket).await;
    "Ok".to_string()
}

async fn spotify_init() -> AuthCodeSpotify {
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

async fn spotify_qr(auth: AuthCodeSpotify) -> String{
    let qr = rspotify::AuthCodeSpotify::get_authorize_url(&auth, false).unwrap();
    //qr_code::Data::new(rspotify::AuthCodeSpotify::get_authorize_url(&self.sp_auth, false).unwrap()).unwrap(); //create login url
    qr
}


async fn startrecasy(correct: bool, tmp_dir: String) -> Song{
    if correct == true {
        std::thread::sleep(std::time::Duration::from_secs(WAIT_WHEN_CORRECT)); //wait 60 if the correct song is found with reasable confidence
    } else {
        std::thread::sleep(std::time::Duration::from_secs(WAIT_REC)); //wait to slow down recognition, i don't want to spam shazam, might not be nesesairy 
    }
    let mut rectime = REC_TIME_S;
    let mut rec: std::result::Result<(), anyhow::Error>;
    let mut tracksong: Result<Song, anyhow::Error> = Ok(Song::default());
    let mut count = 0;

    while tracksong.as_ref().unwrap().track_name == "nosong" && count <= 3 && tracksong.is_ok(){
        count += 1;
        rectime *= 2;
        rec = rec_wav(tmp_dir.clone(), rectime); //record audio
        if rec.is_err(){
            panic!("{}", rec.unwrap_err()); //panic when program fails to record audio
        }
        tracksong = shazamrec(tmp_dir.clone()).await; //try to recognize song
        if tracksong.is_err() {
            let mut ret_error = Song::default();
            ret_error.error = tracksong.err().unwrap().to_string();
            return ret_error
        }
    }
    tracksong.unwrap()

}

async fn shazamrec(tmp_dir: String) -> Result<Song, anyhow::Error> {
    let mut output: std::process::Output = std::process::Output{status: std::process::ExitStatus::default(), stdout: vec![0],stderr: vec![0]}; //init with empty so the compiler does not complain
    // use the right python envirement for windows or linux
    if OS == "windows" {
        output = Command::new("./win-dist-x86_64/ShazamIO/ShazamIO.exe")
            .args([(tmp_dir.clone()+"recorded.wav").as_str()])
//            .args(["ShazamIO.py", "song.wav"])
            .output()?;
    } else if OS == "linux" {
        if ARCHITECTURE == "aarch64" {
            output = Command::new("./lx-dist-aarch64/ShazamIO/ShazamIO")
            .args([(tmp_dir.clone()+"recorded.wav").as_str()])
//            .args(["ShazamIO.py", "song.wav"])
            .output()?;
        } else if ARCHITECTURE == "x86_64" {
            output = Command::new("./lx-dist-x86_64/ShazamIO/ShazamIO")
            .args([(tmp_dir.clone()+"recorded.wav").as_str()])
//            .args(["ShazamIO.py", "song.wav"])
            .output()?;
        }
    }
    let pyerrout = str::from_utf8(&output.stderr).unwrap();
    if pyerrout.is_empty(){
        let jstring = str::from_utf8(&output.stdout)?.to_string();
        println!("song: {}", jstring);
        let shazam_json_p: Value = serde_json::from_str(&jstring).unwrap();
        if !shazam_json_p["track"]["title"].is_string(){ // write No song detected to songname when no song was detected
            let mut nosong = Song::default();
            nosong.track_name = "nosong".to_string();
            Ok(nosong)
        } else { // populate Song whit corect values
            let mut imgpath = "./unknown.png".to_string();
            if !shazam_json_p["track"]["images"]["coverart"].as_str().is_none() { //if image is available
                imgpath = get_image(shazam_json_p["track"]["images"]["coverart"].as_str().unwrap(), tmp_dir.clone() + shazam_json_p["track"]["title"].as_str().unwrap().replace(" ", "_").as_str() + ".jpg" ).await.unwrap();
            } else {
                imgpath = "./unknown.png".to_string();
            }
            
            let mut song = Song::default();
            song.track_name = shazam_json_p["track"]["title"].as_str().unwrap().to_string();
            song.artist_name = shazam_json_p["track"]["subtitle"].as_str().unwrap().to_string();
            song.art = imgpath;
            Ok(song)
        }
    } else{
        let errorout = str::from_utf8(&output.stderr)?.to_owned();
        //println!("Error: {}", errorout);
        Err(anyhow::Error::msg(errorout))
    }
   
}


#[derive(Parser, Debug)]
#[command(version, about = "CPAL record_wav example", long_about = None)]
struct Opt {
    /// The audio device to use
    #[arg(short, long, default_value_t = String::from("default"))]
    device: String,

    /// Use the JACK host
    #[cfg(all(
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd"
        ),
        feature = "jack"
    ))]
    #[arg(short, long)]
    #[allow(dead_code)]
    jack: bool,
}

fn rec_wav(tmp_dir: String, time_s: u64) -> Result<(), anyhow::Error>{
    let opt = Opt::parse();

    // Conditionally compile with jack if the feature is specified.
    #[cfg(all(
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd"
        ),
        feature = "jack"
    ))]
    // Manually check for flags. Can be passed through cargo with -- e.g.
    // cargo run --release --example beep --features jack -- --jack
    let host = if opt.jack {
        cpal::host_from_id(cpal::available_hosts()
            .into_iter()
            .find(|id| *id == cpal::HostId::Jack)
            .expect(
                "make sure --features jack is specified. only works on OSes where jack is available",
            )).expect("jack host unavailable")
    } else {
        cpal::default_host()
    };

    #[cfg(any(
        not(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd"
        )),
        not(feature = "jack")
    ))]
    let host = cpal::default_host();

    // Set up the input device and stream with the default input config.
    let device = if opt.device == "default" {
        host.default_input_device()
    } else {
        host.input_devices()?
            .find(|x| x.name().map(|y| y == opt.device).unwrap_or(false))
    }
    .expect("failed to find input device");

    println!("Input device: {}", device.name()?);

    let config = device
        .default_input_config()
        .expect("Failed to get default input config");
    println!("Default input config: {:?}", config);

    // The WAV file we're recording to.
    let fullpath = (tmp_dir + "recorded.wav").as_str().to_owned();
    let spath: &str = fullpath.as_str();
    let spec = wav_spec_from_config(&config);
    let writer = hound::WavWriter::create(spath, spec)?;
    let writer = Arc::new(Mutex::new(Some(writer)));

    // A flag to indicate that recording is in progress.
    println!("Begin recording...");

    //TODO: split fn here ^ get config once v record 

    // Run the input stream on a separate thread.
    let writer_2 = writer.clone();

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

    let stream = match config.sample_format() {
        cpal::SampleFormat::I8 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i8, i8>(data, &writer_2),
            err_fn,
            None,
        )?,
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i16, i16>(data, &writer_2),
            err_fn,
            None,
        )?,
        cpal::SampleFormat::I32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i32, i32>(data, &writer_2),
            err_fn,
            None,
        )?,
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<f32, f32>(data, &writer_2),
            err_fn,
            None,
        )?,
        sample_format => {
            return Err(anyhow::Error::msg(format!(
                "Unsupported sample format '{sample_format}'"
            )))
        }
    };

    stream.play()?;

    // Let recording go for roughly three seconds.
    std::thread::sleep(std::time::Duration::from_secs(time_s));
    drop(stream);
    writer.lock().unwrap().take().unwrap().finalize()?;
    println!("Recording {} complete!", spath);
    Ok(())
}

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    if format.is_float() {
        hound::SampleFormat::Float
    } else {
        hound::SampleFormat::Int
    }
}

fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> hound::WavSpec {
    hound::WavSpec {
        channels: config.channels() as _,
        sample_rate: config.sample_rate().0 as _,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: sample_format(config.sample_format()),
    }
}

type WavWriterHandle = Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>;

fn write_input_data<T, U>(input: &[T], writer: &WavWriterHandle)
where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T>,
{
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                let sample: U = U::from_sample(sample);
                writer.write_sample(sample).ok();
            }
        }
    }
}

//needs to change name every time
async fn get_image<'a>(link: &'a str,store: String) -> Result<String, anyhow::Error> {
    let target = link;
    let response = reqwest::get(target).await.unwrap().bytes().await.unwrap();
    let image = image::load_from_memory(&response)?;
    image.save(&store)?;
   Ok(store.to_string())
}

//let jstring = shazamrec()?;
    //println!("song: {}", jstring);
    //let shazam_json_p: Value = serde_json::from_str(&jstring).unwrap();
    //let mut song1 = Song { 
    //    track_name: shazam_json_p["track"]["title"].as_str().unwrap().to_string(), 
    //    artist_name: shazam_json_p["track"]["title"].as_str().unwrap().to_string(), 
    //    art: shazam_json_p["track"]["images"]["coverart"].as_str().unwrap().to_string(), 
    //};
    //println!("song: {}", song1.track_name);