use core::str;
use std::env;
use iced::overlay::menu::Catalog;
use iced::{settings, window, Background, Border, Color, Renderer, Shadow, Size, Task, Theme};
//use iced::Subscription;
use iced::Element;
//use iced::time::{self, Duration};
use iced::{widget::{self, button, column, text, row, container, qr_code, stack, opaque,mouse_area,center, Button}, Length, Font, Alignment};
use iced::widget::{image as iceimage, toggler};
//use image::imageops::overlay;
use std::fs::{self, remove_dir_all, create_dir};
use anyhow::Result;
use rspotify::{self, AuthCodeSpotify, Token};
use tokio;

mod song;
use song::Song;
use song::spotify;
use song::rcognize;
use song::spotify::SP_CACHE_PATH;
use song::get_image;

pub static CUSTOM_FONT: Font = Font::with_name("Less Perfect DOS VGA");
pub mod bytes {
    pub static CUSTOM_FONT: &[u8] = include_bytes!("../LessPerfectDOSVGA.ttf");
}

pub static TMP_DIR: &str = "./tmp/"; 
static _EVERY_S: u64 = 3600; //run tick every amount of seconds
static OS: &str = env::consts::OS;
static ARCHITECTURE: &str = env::consts::ARCH;
static TEXT_SIZE: u16 = 50;
static TEXT_COLOR: Color = Color{r: 0.12, g: 0.84, b:0.38, a: 1.0};
static SP_GREEN: Color = Color{r: 0.12, g: 0.84, b:0.38, a: 1.0};
static GRAY_HIGHLIGHT: Color = Color{r: 0.18, g: 0.18, b:0.18, a: 1.0};
static DARK_GRAY: Color = Color{r: 0.08, g: 0.08, b:0.08, a: 1.0};
static BLACK: Color = Color{r: 0.0, g: 0.0, b:0.0, a: 1.0};
static DARK_BLUE: Color = Color{r: 0.12, g: 0.16, b:0.23, a: 1.0};
static BACKGROUND_DARK_BLUE: Background = Background::Color(DARK_BLUE);
static BACKGROUND_GRAY: Background = Background::Color(GRAY_HIGHLIGHT);


fn main() -> Result<(), anyhow::Error> {
    println!("OS: {}", OS);
    println!("Architecture: {}", ARCHITECTURE);
    let path = env::current_dir()?;
    println!("The current directory is {}", path.display());
    if fs::exists(TMP_DIR)? {
        remove_dir_all(TMP_DIR)?;
    }
    create_dir(TMP_DIR)?;
    if !fs::exists("./spotify_cache/")?{
        create_dir("./spotify_cache/")?;
    }
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
    remove_dir_all(TMP_DIR)?;
    Ok(())
}

#[derive(Debug, Clone)]
pub enum DisplayMode{
    Shazam,
    Spotify,
}

#[derive(Debug, Clone)]
enum Message {
    Detect,
    Exit,
    DisplaySong(Result<Song, String>),
    DisplayImg(Result<String, String>),
    Menu(String),
    SpInit,
    SpSaveAuth(AuthCodeSpotify),
    SpShowQr(String),
    SpAuthOk,
    SpAuthError(Result<Token, String>),
    SpModeRun,
    SpModeToggle(bool),
    Demo,
    GetMainWinId,
    StoreMainWinId(window::Id),
    FullscreenExec(window::Id),
    Fullscreen,
    ShowMenu,
    HideMenu,
    SwitchPage(AppPage)
//    Tick,
}

#[derive(Debug)]
struct App{
    song: Song,
    track_name_prev: [String; 5],
    prev_index: usize,
    tmp_dir: String,
    winid: window::Id,
    correct: bool,
    sp_auth: rspotify::AuthCodeSpotify,
    sp_auth_url_data: qr_code::Data,
    sp_init_done: bool,
    sp_mode: bool,
    show_menu: bool,
    page: AppPage,
    sp_callback_handle: Option<tokio::task::JoinHandle<()>>,
    shazamming: bool,
    display_mode: DisplayMode,
}

impl Default for App {
    fn default() -> Self { App::default()}
}

impl App {
    fn default() -> App {
        App{ 
            song: Song::default(),
            track_name_prev: ("Previous Track-name0".to_string(), "Previous Track-name1".to_string(), "Previous Track-name2".to_string(), "Previous Track-name3".to_string(), "Previous Track-name4".to_string()).into(),
            prev_index: 0,
            tmp_dir: TMP_DIR.to_string(),
            winid: window::Id::unique(),
            correct: false,
            sp_auth: AuthCodeSpotify::default(),
            sp_auth_url_data: qr_code::Data::new( "http://localhost/").unwrap(),
            sp_init_done: false,
            sp_mode: false,
            show_menu: false,
            page: AppPage::Main,
            sp_callback_handle: None,
            shazamming: false,
            display_mode: DisplayMode::Shazam,
        }
    }

    fn startup() -> (App, Task<Message>) {
        (App::default(), Task::batch(vec![Task::done(Message::GetMainWinId)]))
    }
    
//    fn songsubscription(&self) -> Subscription<Message>{
//        time::every(Duration::from_secs(EVERY_S)).map(|_| Message::Tick)
//    }

    fn update(&mut self, message: Message) -> Task<Message>{
        self.tmp_dir = TMP_DIR.to_string();
        match message {
            Message::SwitchPage(page) => {
                self.page = page;
                match self.page {
                    AppPage::Main => {
                        match &self.sp_callback_handle {
                            Some(h)=>{
                                h.abort();
                                self.sp_callback_handle = None;
                                if Token::from_cache(SP_CACHE_PATH).is_err(){
                                    self.sp_init_done = false;
                                }
                            }
                            None =>{}
                        }
                        self.show_menu = false;
                        Task::none()
                    }
                    AppPage::Settings => {
                        Task::none()
                    }
                    AppPage::SpotifyLogin => {
                        Task::none()
                    }
                    AppPage::SpotifySettings => {
                        Task::none()
                    }
                }
            }
//            Message::Tick => {
//                iced::Task::perform(rcognize::startrecasy(self.correct.clone(), self.tmp_dir.clone()), Message::DisplaySong)
//            }
            Message::Detect => {
                self.shazamming = true;
                iced::Task::perform(rcognize::startrecasy(self.correct.clone(), self.tmp_dir.clone()), Message::DisplaySong)
            }
            Message::Exit => {
                iced::exit::<Message>()
            },
            Message::DisplaySong(song) => {
                match song {
                    Ok(song) => {
                        if self.song.track_name == song.track_name{
                            match self.display_mode {
                                DisplayMode::Shazam => {
                                    return Task::perform(rcognize::startrecasy(self.correct.clone(), self.tmp_dir.clone()), Message::DisplaySong);
                                }
                                DisplayMode::Spotify => {
                                    return Task::perform(spotify::spotify_get_current(self.sp_auth.clone()), Message::DisplaySong);
                                }
                            }
                        }
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
                            self.song = song.clone();
                        }
                        match self.display_mode {
                            DisplayMode::Shazam => {
                                Task::batch(vec![Task::perform(rcognize::startrecasy(self.correct.clone(), self.tmp_dir.clone()), Message::DisplaySong), Task::perform(get_image(song.art_url, song.track_name.replace(" ", "_") + ".jpg"), Message::DisplayImg)])
                            }
                            DisplayMode::Spotify => {
                                Task::batch(vec![Task::perform(spotify::spotify_get_current(self.sp_auth.clone()), Message::DisplaySong), Task::perform(get_image(song.art_url, song.track_name.replace(" ", "_") + ".jpg"), Message::DisplayImg)])
                            }
                        }
                    }
                    Err(e) => {
                        self.song.track_name = e;
                        Task::none()
                    }
                }
            },
            Message::DisplayImg(img_path) => {
                match img_path {
                    Ok(path) => {
                        self.song.art_path = path;
                    }
                    Err(e) => {
                        self.song.track_name = e;
                    }
                }
                Task::none()
            },
            Message::Menu(item) => {
                if item == "spotify" {
                    Task::done(Message::SpInit)
                } else {
                    Task::none()
                }
            },
            Message::Demo => {
                self.song.track_name = "Track Name".to_string();
                self.song.artist_name = "Artist Name".to_string();
                self.song.art_path = "./unknown.png".to_string();
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
            },
            Message::SpInit => {
                if self.sp_init_done {
                    Task::batch(vec![Task::done(Message::SpAuthOk), Task::done(Message::SwitchPage(AppPage::SpotifyLogin))])
                } else if Token::from_cache(SP_CACHE_PATH).is_ok() {
                    self.sp_init_done = true;
                    Task::perform(spotify::spotify_init(), Message::SpSaveAuth)
                    //Task::done(Message::SwitchPage(AppPage::SpotifySettings))
                } else {
                    Task::perform(spotify::spotify_init(), Message::SpSaveAuth)
                }
            },
            Message::SpShowQr(url) => {
                self.sp_auth_url_data = qr_code::Data::new(url).unwrap();
                self.sp_callback_handle = Some(tokio::spawn(spotify::spotify_callback(self.sp_auth.clone())));
                //tokio::spawn(spotify::spotify_callback(self.sp_auth.clone()));
                self.sp_init_done = true;
                Task::batch(vec![Task::done(Message::SpAuthOk),Task::done(Message::SwitchPage(AppPage::SpotifyLogin))])
            },
            Message::SpAuthOk => {
                Task::perform(spotify::spotify_wait_for_token(self.sp_auth.clone()),Message::SpAuthError)
            },
            Message::SpAuthError(res) => {
                match res {
                    Ok(_token) => {
                        //Task::none()
                        match &self.sp_callback_handle {
                            Some(h) => {
                                h.abort();
                                self.sp_callback_handle = None;
                                if Token::from_cache(SP_CACHE_PATH).is_err(){
                                    self.sp_init_done = false;
                                }
                            }
                            None => {

                            }
                        }
                        Task::batch(vec![Task::perform(spotify::spotify_get_current(self.sp_auth.clone()),Message::DisplaySong), Task::done(Message::SwitchPage(AppPage::Main))])
                    }
                    Err(error) =>{
                        self.song.track_name = error;
                        Task::none()
                    }
                }
            },
            Message::SpSaveAuth(auth) => {
                self.sp_auth = auth;
                if self.sp_init_done {
                    Task::done(Message::SwitchPage(AppPage::SpotifySettings))
                } else {
                    Task::perform(spotify::spotify_qr(self.sp_auth.clone()), Message::SpShowQr)
                }
            },
            Message::SpModeRun => {
                Task::perform(spotify::spotify_get_current(self.sp_auth.clone()), Message::DisplaySong)
            },
            Message::SpModeToggle(t) => {
                match t {
                    true => {
                        self.sp_mode = true;
                        self.display_mode = DisplayMode::Spotify;
                        Task::done(Message::SpModeRun)
                    }
                    false => {
                        self.sp_mode = false;
                        self.display_mode = DisplayMode::Shazam;
                        Task::none()
                    }
                }
            },
            Message::HideMenu => {
                self.show_menu = false;
                Task::none()
            }
            Message::ShowMenu => {
                self.show_menu = true;
                widget::focus_next()
            }
        }
    }

    fn termtheme(&self) -> Theme {
        let terminal: iced::theme::Palette = iced::theme::Palette{
            background: BLACK,
            text: SP_GREEN,
            primary: DARK_GRAY,
            success: Color{r: 0.0, g: 0.5, b: 0.0, a: 1.0},
            danger: Color{r: 0.5, g: 0.0, b: 0.0, a: 1.0},
        };
        Theme::custom("Terminal".to_string(), terminal)
    }
    
    fn btntheme(theme: &Theme, status: button::Status) -> button::Style {
        let textcolor = theme.palette().text;
        match status {
            button::Status::Active => { 
                let style = button::Style {
                    background: Some(Background::Color(DARK_GRAY)),
                    text_color: textcolor,
                    border: Border::default(),
                    shadow: Shadow::default(),
                };
                style
            }
            _ => {
                let style = button::Style {
                    background: Some(Background::Color(GRAY_HIGHLIGHT)),
                    text_color: textcolor,
                    border: Border::default(),
                    shadow: Shadow::default(),
                };
                style
            }
            //button::primary(theme, status),
        }
    }

    fn menu_style( _theme: &Theme) -> container::Style {
        container::Style { 
            text_color: Some(SP_GREEN), 
            background: Some(Background::Color(DARK_GRAY)), 
            ..Default::default() 
        }
    }
    fn padded_button<Message: Clone>(label: &str,pad_x: u16, pad_y: u16) -> Button<'_, Message> {
        button(text(label)).padding([pad_y, pad_x])
    }

    fn view(&self) -> Element<Message>{
        //multi use widgers
        let home = button("return")
            .on_press(Message::SwitchPage(AppPage::Main))
            .style(Self::btntheme);

        // main window widgets
        let detect = button("detect")
            .on_press(Message::Detect)
            .style(Self::btntheme);
        let demo = button("demo")
            .on_press(Message::Demo)
            .style(Self::btntheme);
        let exit: Button<'_, Message> = button("exit")
            .on_press(Message::Exit)
            .style(Self::btntheme);
        let menu:Button<'_, Message> = button("menu")
            .on_press(Message::ShowMenu)
            .style(Self::btntheme);
        let fullscreen = button("Fullscreen")
            .on_press(Message::Fullscreen)
            .style(Self::btntheme);
        
        let trackname = text(self.song.track_name.clone())
            .size(TEXT_SIZE)
            .center();
        let artistname= text(self.song.artist_name.clone())
            .size(TEXT_SIZE - 15)
            .center();

        let coverart = iceimage(self.song.art_path.clone())
            .width(300);

        let spotify = Self::padded_button("spotify", 40, 20)
            .on_press(Message::SpInit)
            .style(Self::btntheme);
        let settings = Self::padded_button("settings", 40, 20)
            .on_press(Message::Demo)
            .style(Self::btntheme);
        

        //spotify login widgets
        let sp_back = button("cancel")
        .on_press(Message::SwitchPage(AppPage::Main))
        .style(Self::btntheme);
        let spotify_qr_code = qr_code(&self.sp_auth_url_data);

        //spotify settings widgets
        let spotify_mode = toggler(self.sp_mode)
            .label("Enable spotify mode")
            .on_toggle(Message::SpModeToggle);


        let main_page = container(
            column![
            row![ column![ row![ detect,exit,demo,fullscreen ] ].padding(5).width(Length::FillPortion(2)),column![ menu ].padding(5).align_x(Alignment::End).width(Length::FillPortion(1))],
            row![ column![ trackname, artistname ].padding(40).width(Length::FillPortion(6)).align_x(Alignment::Start), column![coverart].align_x(Alignment::End).width(Length::FillPortion(4)),].height(Length::Fill),
            //row![ spotify_qr_code ],
        ].height(Length::Fill));
        
        match self.page {
            AppPage::Main => {
                if self.show_menu {
                    let menu = container(
                column![ spotify,
                                  settings,
                            ]);
                    sidebar(main_page, menu, Message::HideMenu)
                } else {
                    main_page.into()
                }
            }
            AppPage::Settings => {
                main_page.into()
            }
            AppPage::SpotifyLogin => {
                let sp_login_page = column![
                    row![ sp_back ],
                    row![ spotify_qr_code ]
                ];
                sp_login_page.into()
            }
            AppPage::SpotifySettings => {
                let sp_settings_page = column![
                    spotify_mode,
                    home,
                ]
                .align_x(Alignment::Center)
                .width(Length::Fill)
                .padding(20);
                sp_settings_page.into()
            }
        }
        
    }
    
}

#[derive(Debug, Clone)]
enum AppPage {
    Main,
    Settings,
    SpotifyLogin,
    SpotifySettings,
}

fn sidebar<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        base.into(),
        opaque(
            mouse_area(  
                container( container( column![ opaque(content) ].height(Length::Fill) ).style(App::menu_style) )
                .align_right(Length::Fill)
                .height(Length::Fill)
                .style(|_theme| {
                container::Style {
                    background: Some(
                        Color {
                            a: 0.8,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..container::Style::default()
                }
            }))
            .on_press(on_blur)
        )
    ].height(Length::Fill)
    .into()
}

