use audioinfo::AudioInfo;
use clap::{crate_version, Arg, ArgAction, Command};
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    let matches = Command::new("AudioInfo Generator")
        .version(crate_version!())
        .about("Generates an audioinfo file for the given directory")
        .arg(
            Arg::new("directory")
                .help("Sets the directory to scan for FLAC files")
                .required(true),
        )
        .arg(Arg::new("output").help("Sets the output directory for audioinfo"))
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Enables verbose (debug) output"),
        )
        .arg(
            Arg::new("print")
                .short('p')
                .long("print")
                .action(ArgAction::SetTrue)
                .help("Print AudioInfo to std"),
        )
        .get_matches();

    let directory = matches.get_one::<String>("directory").expect("required");
    let output = matches.get_one::<String>("output");
    let print = matches.get_flag("print");

    let verbose = matches.get_flag("verbose");
    if verbose {
        tracing::subscriber::set_global_default(
            tracing_subscriber::FmtSubscriber::builder()
                .with_max_level(tracing::Level::DEBUG)
                .finish(),
        )
        .expect("Failed to set global default tracing subscriber");
    }
    let audio_info_string = AudioInfo::generate_audio_info_from_path(directory.to_string());

    if print {
        print!("{:}", audio_info_string);
    } else {
        match output {
            Some(output) => {
                save_file(output.to_string(), audio_info_string);
            }
            None => {
                save_file(String::from("./audioinfo.txt"), audio_info_string);
            }
        }
    }
}

fn save_file(path: String, audio_info_string: String) {
    let audioinfo_file_path = Path::new(&path);
    if let Ok(mut file) = fs::File::create(audioinfo_file_path) {
        if let Err(e) = file.write_all(audio_info_string.as_bytes()) {
            println!("Error writing to audioinfo file: {}", e);
        }
    } else {
        println!("Error creating audioinfo file");
    }
}
