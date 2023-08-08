use ngrammatic::{CorpusBuilder, Pad};
use rodio::{Decoder, OutputStream, Sink};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{stdin, BufReader};
use std::path::PathBuf;

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

fn show_library(library: &HashMap<String, PathBuf>) {
    for (key, _) in library {
        println!("{key}");
    }
}

fn main() {
    // Initialize library hashmap
    let mut library = HashMap::new();

    let welcome_string: &str = "Welcome\n
        1 - Play a song
        2 - View songs 
        3 - Help 
        4 - Exit";

    load_library(&mut library);

    println!("{}", welcome_string);
    // main loop
    loop {
        let mut input = String::new();

        stdin().read_line(&mut input).expect("[!] Invalid input");

        match input.as_str().trim_end() {
            "1" => play_song(&library),
            "2" => show_library(&library),
            "3" => println!("{}", welcome_string),
            "4" => break,
            _ => println!("[!] Invalid option"),
        }
    }
}
