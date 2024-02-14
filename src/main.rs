use minimp3::Decoder as dc;
use minimp3::{Error, Frame};
use ngrammatic::{CorpusBuilder, Pad};
use reqwest::{self};
use rodio::{Decoder, OutputStream, Sink};
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{stdin, BufReader};
use std::path::PathBuf;
use std::process::Command;

const YOUTUBE_SEARCH: &str =
    "https://youtube.googleapis.com/youtube/v3/search?part=snippet&part=id&q=";
const DOWNLOAD_LOC: &str = "/Users/Ryan/soft/music-viz/library/";
const YOUTUBE_URL: &str = "https://www.youtube.com/watch?v=";

// Searches the library for a song name using fuzzy search
fn search_library(library: &mut HashMap<String, PathBuf>) -> PathBuf {
    load_library(library);

    // build corpus for searching
    let mut corpus = CorpusBuilder::new().arity(2).pad_full(Pad::Auto).finish();

    // Aggregate names of songs into corpus
    for (key, _) in &mut *library {
        corpus.add_text(key);
    }

    // get input
    let mut input = String::new();
    println!("Enter search");
    loop {
        stdin().read_line(&mut input).expect("[!] Invalid input");
        let results = corpus.search(&input, 0.10);
        if let Some(result) = results.first() {
            println!("Closest match: {}", result.text);
            println!("Play this song? (y/n)");
            input.clear();
            stdin().read_line(&mut input).expect("[!] Invalid input");
            match input.as_str().trim_end() {
                "y" => {
                    return library
                        .get(&result.text)
                        .expect("[!] Result not found in library")
                        .clone();
                }
                "Y" => {
                    return library
                        .get(&result.text)
                        .expect("[!] Result not found in library")
                        .clone();
                }
                _ => {
                    println!("Try a new search");
                    continue;
                }
            }
        } else {
            println!("[!] No word similar to {input} found in library");
        }
    }
}

// Plays a song
fn play_song(library: &mut HashMap<String, PathBuf>) {
    let song: PathBuf = search_library(library);
    // get output stream handle to sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    // load sound from file, using path from Cargo.toml
    let file = BufReader::new(File::open(song.as_path()).unwrap());
    // decode sound file into a source
    let source = Decoder::new(file).unwrap();
    // Play sound on device
    //let _ = stream_handle.play_raw(source.convert_samples());
    sink.append(source);
    // keep thread alive
    //std::thread::sleep(std::time::Duration::from_secs(253));
    sink.sleep_until_end();
}

// Loads hashmap with key value pairs: song name, path to file from library folder
fn load_library(library: &mut HashMap<String, PathBuf>) {
    for entry in fs::read_dir("/Users/Ryan/soft/music-viz/library/")
        .expect("[!] Failed to read library failed")
    {
        let entry = entry.expect("[!] Couldn't read item in library directory");
        let path = entry.path();
        let mut name = entry
            .file_name()
            .into_string()
            .expect("[!] String conversion failed");
        name.truncate(name.len() - 4);
        library.insert(name, path);
    }
}

// Displays the library for the user
fn show_library(library: &mut HashMap<String, PathBuf>) {
    load_library(library);

    for (key, _) in library {
        println!("{key}");
    }
}

// Sends search request to youtube api and returns user-picked result
fn send_request(search_string: String) -> String {
    // format search string
    let formatted_string = search_string.replace(" ", "%20");

    println!("Searching...");

    let body = reqwest::blocking::get(YOUTUBE_SEARCH.to_owned() + &formatted_string + API_KEY)
        .expect("[!] request faileld")
        .json::<serde_json::Value>()
        .expect("[!] Reading response failed");

    let mut id_vec = Vec::new();

    for i in 0..5 {
        id_vec.push(body["items"][i]["id"]["videoId"].clone().to_string());
        println!("{} - {}", i + 1, body["items"][i]["snippet"]["title"]);
    }

    println!("\nSelect 1-5");
    let mut input = String::new();
    let mut choice: usize;

    loop {
        input.clear();
        stdin().read_line(&mut input).expect("[!] Invalid input");
        choice = input.as_str().trim_end().parse().unwrap();

        if choice > 5 || choice < 1 {
            println!("Invalid number, try again");
            continue;
        }
        break;
    }

    // minus 1 to reflect real index
    return id_vec[choice - 1].clone();
}

// Pulls song from youtube in mp3 form and adds it to library
fn download_song() {
    let mut input = String::new();
    println!("Search Youtube:");
    stdin().read_line(&mut input).expect("[!] Invalid input");
    // search
    let song_id: String = send_request(input).replace("\"", "");
    let mut input = String::new();

    println!("Set song name:");
    stdin().read_line(&mut input).expect("[!] Invalid input");

    println!("Downloading...");
    Command::new("yt-dlp")
        .arg(YOUTUBE_URL.to_owned() + &song_id)
        .arg("-x")
        .arg("--audio-format")
        .arg("mp3")
        .arg("--output")
        .arg(DOWNLOAD_LOC.to_owned() + &input.trim_end() + ".mp3")
        .spawn()
        .expect("[!] yt-dlp failed to execute");
}

fn test_fft() {
    let mut decoder = dc::new(File::open("/Users/Ryan/soft/music-viz/library/myeyes.mp3").unwrap());

    loop {
        match decoder.next_frame() {
            Ok(Frame {
                data,
                sample_rate,
                channels,
                ..
            }) => {
                //println!("Decoded {} samples", data.len() / channels);
                //println!("Data: {:?}", data);
                if data[0] > 40 {
                    println!("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
                } else {
                    for _ in 0..data[0] {
                        print!("x");
                    }
                }
                println!("");
            }
            Err(Error::Eof) => break,
            Err(e) => panic!("{:?}", e),
        }
    }
}

fn main() {
    // Initialize library hashmap
    let mut library = HashMap::new();

    let welcome_string: &str = "Welcome\n
        1 - Play song from library
        2 - View songs 
        3 - Download song
        4 - Help 
        5 - Exit";

    load_library(&mut library);

    println!("{}", welcome_string);
    // main loop
    loop {
        let mut input = String::new();

        stdin().read_line(&mut input).expect("[!] Invalid input");

        match input.as_str().trim_end() {
            "1" => play_song(&mut library),
            "2" => show_library(&mut library),
            "3" => download_song(),
            "4" => println!("{}", welcome_string),
            "5" => break,
            "6" => test_fft(),
            _ => println!("[!] Invalid option"),
        }
    }
}
