use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, Sink};
use rodio::source::{SineWave, Source};

fn main() {
    // get output stream handle to sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    // load sound from file, using path from Cargo.toml
    let file = BufReader::new(File::open("/Users/Ryan/Software/music-viz/src/myeyes.mp3").unwrap());
    // decode sound file into a source 
    let source = Decoder::new(file).unwrap();
    // Play sound on device 
    //let _ = stream_handle.play_raw(source.convert_samples());
    sink.append(source);
    // keep thread alive
    //std::thread::sleep(std::time::Duration::from_secs(253));
    sink.sleep_until_end();
}
