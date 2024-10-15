use core::str;
use std::env;
use iced::{settings, window, Background, Border, Color, Shadow, Size, Task, Theme};
//use iced::Subscription;
use iced::Element;
//use iced::time::{self, Duration};
use iced::{widget::{button, column, text, row, container, qr_code}, Length, Font, Alignment};
use iced::widget::image as iceimage;
use std::fs::{self, remove_dir_all, create_dir};
use anyhow::Result;
use rspotify::{self, AuthCodeSpotify, Token};
use tokio;

mod song;
use song::Song;
use song::spotify;
use song::rcognize;


pub static CUSTOM_FONT: Font = Font::with_name("Less Perfect DOS VGA");
pub mod bytes {
    pub static CUSTOM_FONT: &[u8] = include_bytes!("../LessPerfectDOSVGA.ttf");
}

pub static TMP_DIR_S: &str = "./tmp/"; 
static _EVERY_S: u64 = 3600; //run tick every amount of seconds
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
//    Tick,
//    None(()),
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
        (App::default(), Task::done(Message::GetMainWinId))
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
    
//    fn songsubscription(&self) -> Subscription<Message>{
//        time::every(Duration::from_secs(EVERY_S)).map(|_| Message::Tick)
//    }
    
    fn update(&mut self, message: Message) -> Task<Message>{
        self.tmp_dir = TMP_DIR_S.to_string();
        match message {
//            Message::Tick => {
//                iced::Task::perform(rcognize::startrecasy(self.correct.clone(), self.tmp_dir.clone()), Message::DisplaySong)
//            }
            Message::Detect => {
                iced::Task::perform(rcognize::startrecasy(self.correct.clone(), self.tmp_dir.clone()), Message::DisplaySong)
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
                    iced::Task::perform(rcognize::startrecasy(self.correct.clone(), self.tmp_dir.clone()), Message::DisplaySong)
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
//            Message::None(n) => {
//                Task::none()
//            },
            Message::SpShowQr(url) => {
                self.sp_auth_url_data = qr_code::Data::new(url).unwrap();
                tokio::spawn(spotify::spotify_callback(self.sp_auth.clone()));
                Task::done(Message::SpAuthOk)
            },
            Message::SpAuthOk => {
                Task::perform(spotify::spotify_wait_for_token(self.sp_auth.clone()),Message::SpAuthError)
            },
            Message::SpAuthError(res) => {
                match res {
                    Ok(token) => {
                        Task::perform(spotify::spotify_get_current(token),Message::DisplaySong)
                    }
                    Err(error) =>{
                        self.track_name = error;
                        Task::none()
                    }
                }
            },
            Message::SpInit => {
                Task::perform(spotify::spotify_init(), Message::SpSaveAuth)
            },
            Message::SpSaveAuth(auth) => {
                self.sp_auth = auth;
                Task::perform(spotify::spotify_qr(self.sp_auth.clone()), Message::SpShowQr)
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
        
        //let spotify_qr_code = qr_code(&self.sp_auth_url_data);

        container(
            column![
            row![ column![ row![ detect,exit,demo,fullscreen ] ].padding(5).width(Length::FillPortion(2)),column![ menu ].padding(5).align_x(Alignment::End).width(Length::FillPortion(1))],
            row![ column![ trackname, artistname ].padding(40).width(Length::FillPortion(6)).align_x(Alignment::Start), column![coverart].align_x(Alignment::End).width(Length::FillPortion(4)),],
            //row![ spotify_qr_code ],
        ]).into()
    }
    
}


