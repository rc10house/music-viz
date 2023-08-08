use ngrammatic::{CorpusBuilder, Pad};
use reqwest::{self, Response};
use rodio::{Decoder, OutputStream, Sink};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{stdin, BufReader};
use std::path::PathBuf;

const YOUTUBE_SEARCH: &str =
    "https://youtube.googleapis.com/youtube/v3/search?part=snippet&part=id&q=";
const API_KEY: &str = "&key=AIzaSyCIVk8DHCPmnOEuTbxol_S_a9DeJcVQM68";

// Searches the library for a song name using fuzzy search
fn search_library(library: &HashMap<String, PathBuf>) -> PathBuf {
    // build corpus for searching
    let mut corpus = CorpusBuilder::new().arity(2).pad_full(Pad::Auto).finish();

    // Aggregate names of songs into corpus
    for (key, _) in library {
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
            println!("{}", input);
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
fn play_song(library: &HashMap<String, PathBuf>) {
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
    println!("HashMap: {:?}", library);
}

// Displays the library for the user
fn show_library(library: &HashMap<String, PathBuf>) {
    for (key, _) in library {
        println!("{key}");
    }
}

fn send_request(search_string: String) {
    // format search string
    let formatted_string = search_string.replace(" ", "%20");

    let body = reqwest::blocking::get(YOUTUBE_SEARCH.to_owned() + &formatted_string + API_KEY)
        .expect("[!] request faileld")
        .json::<serde_json::Value>()
        .expect("[!] Reading response failed");

    for i in 0..5 {
        println!(
            "{} - {} - {}",
            i, body["items"][i]["snippet"]["title"], body["items"][i]["id"]["videoId"]
        );
    }
}

// Pulls song from youtube in mp3 form and adds it to library
fn download_song(library: &mut HashMap<String, PathBuf>) {
    let mut input = String::new();
    println!("Search Youtube:");
    stdin().read_line(&mut input).expect("[!] Invalid input");
    send_request(input);
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
            "1" => play_song(&library),
            "2" => show_library(&library),
            "3" => download_song(&mut library),
            "4" => println!("{}", welcome_string),
            "5" => break,
            _ => println!("[!] Invalid option"),
        }
    }
}
