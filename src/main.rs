use core::str;
use std::process::Command;
use std::str::Utf8Error;
use std::env;
use iced::{Font, Subscription, Task};
use iced::time::{self, Duration, Instant};
use serde_json::Value;
use iced::{widget::{button, column, text, row, Column, Row}, Length, Settings, font, Alignment};
use iced::widget::Image as IceImage;
use iced::widget::image as iceimage;
use hound;
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use tempfile::{tempdir, TempDir};
use std::fs::{self, remove_file, remove_dir_all, create_dir, File};
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use anyhow::Result;
use std::thread::{self, Thread};

static TMP_DIR_S: &str = "./tmp/";
static REC_TIME_S: u64 = 20;
static EVERY_S: u64 = 600;
static OS: &str = env::consts::OS;


fn main() -> Result<(), anyhow::Error> {
    println!("OS: {}", OS);
    let path = env::current_dir()?;
    println!("The current directory is {}", path.display());
    if fs::exists(TMP_DIR_S)? {
        remove_dir_all(TMP_DIR_S)?;
    }
    create_dir(TMP_DIR_S)?;
    iced::application("ShazamDisplay", Song::update, Song::view).subscription(Song::songsubscription).run()?;
    iced::run("start", Song::update, Song::view)?;
    remove_dir_all(TMP_DIR_S)?;
    Ok(())
}

#[derive(Clone, Debug)]
enum Message {
    Detect,
    Exit,
    DisplaySong(Song),
    Tick,
}

#[derive(Default, Debug, Clone)]
struct Song{
    track_name: String,
    artist_name: String,
    art: String,
    tmps: String,
}

impl Song {
    fn songsubscription(&self) -> Subscription<Message>{
        time::every(Duration::from_secs(EVERY_S)).map(|_| Message::Tick)
    }
    fn update(&mut self, message: Message) -> Task<Message>{
        self.tmps = TMP_DIR_S.to_string();
        match message {
            Message::Tick => {
                iced::Task::perform(startrecasy(self.clone()), Message::DisplaySong)
            }
            Message::Detect => {
                iced::Task::perform(startrecasy(self.clone()), Message::DisplaySong)
            //old tread code
                //let tempsongtop = Arc::new(Mutex::new(Song::default()));
//
                //let tempsong = Arc::clone(&tempsongtop);
                //let rechandle = thread::spawn(move ||{
                //    //let tempsong = tempsongtop.clone();
                //    let s = startrec();
                //    if s.track_name == Song::default().track_name{
                //        tempsong.lock().unwrap().track_name = "No song detected.".to_string();
                //    } else {
                //        tempsong.lock().unwrap().track_name = s.track_name;
                //        tempsong.lock().unwrap().artist_name = s.artist_name;
                //        tempsong.lock().unwrap().art = s.art;
                //    }
                //});
                //rechandle.join();
                //self.track_name = tempsongtop.lock().unwrap().track_name.clone();
                //self.artist_name = tempsongtop.lock().unwrap().artist_name.clone();
                //self.art = tempsongtop.lock().unwrap().art.clone();

            //old sequential code
                //let res = rec_wav();
                //if res.is_err(){
                //    panic!("PANIC: Failed to record sound")
                //}
                //let trackres = shazamrec();
                //if trackres.is_ok(){
                //    let track = trackres.unwrap();
                //    if track.track_name == Song::default().track_name{
                //        self.track_name = "No song detected.".to_string();
                //    } else {
                //        self.track_name = track.track_name;
                //        self.artist_name = track.artist_name;
                //        self.art = track.art;
                //    }
                //} else {
                //    self.track_name = "error".to_string();
                //}
            }
            Message::Exit => {
                panic!();
                let id = iced::window::Id::unique();
                let _task: Task<()> = iced::window::close(id);
                Task::none()
            },
            Message::DisplaySong(song) => {
                self.track_name = song.track_name;
                self.artist_name = song.artist_name;
                if self.track_name == "No song detected".to_string(){
                    self.art = "".to_string();
                } else {
                    self.art = song.art.clone();
                }
                self.view();
                Task::none()
            },
        }
    }

    fn view(&self) -> Column<Message>{
        let titlefont = font::Font{family: font::Family::Name("Roboto"), weight: font::Weight::Medium, stretch: font::Stretch::Normal, style: font::Style::Normal};
        let artistfont = font::Font{family: font::Family::Name("Roboto"), weight: font::Weight::Normal, stretch: font::Stretch::Normal, style: font::Style::Normal};
        
        let detect = button("detect").on_press(Message::Detect);
        let exit = button("exit").on_press(Message::Exit);

        let trackname = text(self.track_name.clone())
            .font(titlefont)
            .size(40);
        let artistname= text(self.artist_name.clone())
            .font(artistfont)
            .size(30);

        let coverart = iceimage(self.art.clone())
            .width(300);

        let interface = column![
            column![ row![ detect, exit ].padding(5)],
            column![ row![ column![trackname, artistname].padding(40).width(600), coverart ].align_y(Alignment::Center),]
                .align_x(Alignment::Center)
                .width(Length::Fill),
            ];
        interface
    }
}

async fn startrecasy(s: Song) -> Song{
    let res = rec_wav(s.clone());
    if res.is_err(){
        panic!("{}", res.unwrap_err());
    }
    let trackres = shazamrec(s);
    if trackres.is_ok(){
        trackres.unwrap()
    } else {
        println!("{:?}",trackres);
        let mut songerror = Song::default();
        songerror.artist_name = "error".to_string();
        songerror
    }
}

fn shazamrec(s: Song) -> Result<Song, anyhow::Error> {
    let output: std::process::Output;
    if OS == "windows" {
        output = Command::new("./win-py-venv/Scripts/python.exe")
            .args(["ShazamIO.py", (s.tmps.clone()+"recorded.wav").as_str()])
//            .args(["ShazamIO.py", "song.wav"])
            .output()?;
    } else {
        output = Command::new("./lx-py-venv/bin/python")
            .args(["ShazamIO.py", (s.tmps.clone()+"recorded.wav").as_str()])
//            .args(["ShazamIO.py", "song.wav"])
            .output()?;
    }
    let pyerrout = str::from_utf8(&output.stderr).unwrap();
    if pyerrout.is_empty(){
        let jstring = str::from_utf8(&output.stdout)?.to_string();
        println!("song: {}", jstring);
        let shazam_json_p: Value = serde_json::from_str(&jstring).unwrap();
        if !shazam_json_p["track"]["title"].is_string(){
            let mut nosong = Song::default();
            nosong.artist_name = "No song detected".to_string();
            Ok(nosong)
        } else {
            let imgpath = get_image(shazam_json_p["track"]["images"]["coverart"].as_str().unwrap(), s.tmps.clone() + shazam_json_p["track"]["title"].as_str().unwrap().replace(" ", "_").as_str() + ".jpg" )?;
            let song = Song{
                track_name: shazam_json_p["track"]["title"].as_str().unwrap().to_string(),
                artist_name: shazam_json_p["track"]["subtitle"].as_str().unwrap().to_string(),
                art: imgpath,
                tmps: s.tmps,
            };
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

fn rec_wav(s: Song) -> Result<(), anyhow::Error>{
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
    let fullpath = (s.tmps + "recorded.wav").as_str().to_owned();
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
    std::thread::sleep(std::time::Duration::from_secs(REC_TIME_S));
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
fn get_image<'a>(link: &'a str,store: String) -> Result<String, anyhow::Error> {
    let target = link;
    let response = reqwest::blocking::get(target)?.bytes()?;
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