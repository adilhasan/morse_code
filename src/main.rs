use std::thread::sleep;
use std::{path::PathBuf};
use structopt::StructOpt;
use serde::{Deserialize};
use std::error::Error;
use std::collections::HashMap;
use std::time;
use std::fs::File;
use std::io::{self, BufReader};
use rodio::{Decoder, OutputStream, Sink, source::{Source}};

#[derive(Debug, Deserialize)]
pub struct MorseCode {
    letter: String,
    code: String,
}

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short, long)]
    code_book: Option<PathBuf>,
}

// Read in a word and use the morse-code codebook to convert the word into morse code and sounds.
fn main() -> Result<(), Box <dyn Error>> {

    let opt = Opt::from_args();

    // Read in the message from the commandline
    println!("Enter a message to encode in Morse [alphabetic letters only]:");
    let mut message = String::new();
    io::stdin().read_line(&mut message)?;

    // Get the codebook
    let code_book: PathBuf = match opt.code_book {
        Some(p) => p,
        None => PathBuf::from("./src/morse-codebook.json"),
    };

    let file_in = std::fs::File::open(code_book).expect("unable to open file");
    // Used serde to parse the contents of the JSON file
    let records: Vec<MorseCode> = serde_json::from_reader(file_in).expect("cannot parse json");

    // Get the output stream
    // Get a output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // Load a sound from a file, using a path relative to Cargo.toml
    let file_dot = BufReader::new(File::open("./src/morse-dot5.mp3").unwrap());
    let file_dash = BufReader::new(File::open("./src/morse-dash5.mp3").unwrap());

    // Decode that sound file into a source we need to use buffered so we can clone the sound in the loop
    let source_dot = Decoder::new(file_dot).unwrap();
    let source_dot_buffered = source_dot.buffered();
    let source_dash = Decoder::new(file_dash).unwrap();
    let source_dash_buffered = source_dash.buffered();
    
    
    // Store the code in a hash
    let mut code_lookup = HashMap::new();
    for a_record in records {
        code_lookup.insert(a_record.letter, a_record.code);
    }

    // Loop over the input string and convert to code
    let mut code_message = String::new();
    for a_char in message.chars() {
        if a_char.is_alphabetic() {
            for a_bleep in code_lookup[&a_char.to_string()].chars() {
                if a_bleep == '.' {
                    sink.append(source_dot_buffered.clone());
                } else if a_bleep == '-' {
                    sink.append(source_dash_buffered.clone());
                }
            }
            code_message.push_str(&code_lookup[&a_char.to_string()]);
            code_message.push_str(" ");
        } else if a_char == ' ' {
            sleep(time::Duration::from_secs(2));
            code_message.push_str("/");
        }
    }
    
    // The sound plays in a separate audio thread,
    // so we need to keep the main thread alive while it's playing.
    sink.sleep_until_end();

    println!("Morse code message: /{}/", code_message);
    
    Ok(())
}
