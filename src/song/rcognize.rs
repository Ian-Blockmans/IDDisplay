use std::env;
use std::process::Command;
use core::str;
use serde_json::Value;
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::BufWriter;
use tokio;

use super::Song;
use super::get_image;

static REC_TIME_S: u64 = 3;
static WAIT_WHEN_CORRECT: u64 = 5;
static WAIT_REC: u64 = 1; //wait to slow down recognition, i don't want to spam shazam, might not be necessairy 
static OS: &str = env::consts::OS;
static ARCHITECTURE: &str = env::consts::ARCH;

pub async fn startrecasy(correct: bool, tmp_dir: String) -> Result<Song, String>{
    if correct == true {
        tokio::time::sleep(std::time::Duration::from_secs(WAIT_WHEN_CORRECT)).await; //wait 60 if the correct song is found with reasable confidence
    } else {
        tokio::time::sleep(std::time::Duration::from_secs(WAIT_REC)).await; //wait to slow down recognition, i don't want to spam shazam, might not be nesesairy 
    }
    let tmpdir = tmp_dir.clone();
    let mut rectime = REC_TIME_S;
    let mut rec: std::result::Result<(), anyhow::Error>;
    let mut tracksong: Result<Song, anyhow::Error> = Ok(Song::default());
    let mut count = 0;

    while tracksong.as_ref().unwrap().track_name == "nosong" && count <= 3 && tracksong.is_ok(){
        count += 1;
        rectime *= 2;
        
        rec = rec_wav(tmpdir.clone(), rectime).await; //record audio
        if rec.is_err(){
            panic!("{}", rec.unwrap_err()); //panic when program fails to record audio
        }
        tracksong = shazamrec(tmp_dir.clone()).await; //try to recognize song
        if tracksong.is_err() {
            return Err(tracksong.err().unwrap().to_string());
        }
        tokio::time::sleep(std::time::Duration::from_micros(1)).await; //exit point for tokio
    }
    Ok(tracksong.unwrap())

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
        //println!("song: {}", jstring);
        let shazam_json_p: Value = serde_json::from_str(&jstring).unwrap();
        if !shazam_json_p["track"]["title"].is_string(){ // write No song detected to songname when no song was detected
            let mut nosong = Song::default();
            nosong.track_name = "nosong".to_string();
            Ok(nosong)
        } else { // populate Song whit corect values
            let imgurl;
            if !shazam_json_p["track"]["images"]["coverart"].as_str().is_none() { //if image is available
                imgurl = shazam_json_p["track"]["images"]["coverart"].as_str().unwrap();
                //imgpath = get_image(shazam_json_p["track"]["images"]["coverart"].as_str().unwrap(), shazam_json_p["track"]["title"].as_str().unwrap().replace(" ", "_") + ".jpg" ).await.unwrap();
            } else {
                imgurl = "";
            }
            
            let mut song = Song::default();
            song.track_name = shazam_json_p["track"]["title"].as_str().unwrap().to_string();
            song.artist_name = shazam_json_p["track"]["subtitle"].as_str().unwrap().to_string();
            song.art_url = imgurl.to_string();
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

async fn rec_wav(tmp_dir: String, time_s: u64) -> Result<(), anyhow::Error>{
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

    //println!("Input device: {}", device.name()?);

    let config = device
        .default_input_config()
        .expect("Failed to get default input config");
    //println!("Default input config: {:?}", config);

    // The WAV file we're recording to.
    let fullpath = (tmp_dir + "recorded.wav").as_str().to_owned();
    let spath: &str = fullpath.as_str();
    let spec = wav_spec_from_config(&config);
    let writer = hound::WavWriter::create(spath, spec)?;
    let writer = Arc::new(Mutex::new(Some(writer)));

    // A flag to indicate that recording is in progress.
    //println!("Begin recording...");

    //TODO: split fn here ^ get config once v record 

    // Run the input stream on a separate thread.
    let writer_2 = writer.clone();

    let err_fn = move |err| {
        //eprintln!("an error occurred on stream: {}", err);
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
    //tokio::time::sleep(std::time::Duration::from_secs(time_s)).await;

    drop(stream);
    writer.lock().unwrap().take().unwrap().finalize()?;
    //println!("Recording {} complete!", spath);
    
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

