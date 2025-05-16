use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::exit;

use serde::Deserialize;
use thiserror::Error;
use rusqlite::Error;

use mc_classic;

mod convert;

const INPUT_FOLDER: &str = "input";
const INPUT_FILE: &str = "level.dat";
const OUTPUT_MODE: u8 = 0;
const OUTPUT_FOLDER: &str = "output";
const OUTPUT_FILE: &str = "localStorage.js";
const OUTPUT_WEBSITE: &str = "https://classic.minecraft.net";

#[derive(Deserialize, Debug)]
struct Config {
    input_settings: Input,
    output_settings: Output
}

#[derive(Deserialize, Debug)]
struct Input {
    input_folder: String,
    input_file: String
}

#[derive(Deserialize, Debug)]
struct Output {
    output_mode: u8,
    output_folder: String,
    output_file: String,
    output_website: String
}

#[derive(Error, Debug)]
pub enum GeneralError {
    #[error("Error Parsing Config")]
    TOMLError(#[from] toml::de::Error),

    #[error("File Error")]
    FileError(#[from] std::io::Error),

    #[error("Classic Error")]
    ClassicError(#[from] mc_classic::ClassicError),

    #[error("Conversion Error")]
    ConversionError(#[from] convert::ConversionError),

    #[error("Write Error")]
    WriteError(#[from] rusqlite::Error),

    #[error("Could not find {0}")]
    MissingFile(String),

    #[error("Output mode invalid, expected 0 or 1 but found {0}")]   
    InvalidMode(u8)    
}

fn main () {

    if !fs::exists("config.toml").unwrap() {
        if let Err(e) = build_settings() {throw(e)}
    }

    let conf = fs::read_to_string("config.toml").unwrap().replace("-", "_");
    let config: Config = match toml::from_str(&conf) {
        Ok(c) => c,
        Err(e) => {
            throw(GeneralError::TOMLError(e));
            exit(1)
        }
    };

    if !fs::exists(&config.input_settings.input_folder).unwrap() {
        if let Err(e) = fs::create_dir(&config.input_settings.input_folder) {throw(GeneralError::FileError(e))}
    }
    if !fs::exists(&config.output_settings.output_folder).unwrap() {
        if let Err(e) = fs::create_dir(&config.output_settings.output_folder) {throw(GeneralError::FileError(e))}
    }

    println!("Loading level");
    if !fs::exists(config.input_settings.input_folder.clone() + "/" + &config.input_settings.input_file).unwrap() {
        throw(GeneralError::MissingFile(config.input_settings.input_folder.clone() + "/" + &config.input_settings.input_file));
    }
    let classic: mc_classic::Level = match mc_classic::read_level(config.input_settings.input_folder.clone() + "/" + &config.input_settings.input_file) {
        Ok(c) => c,
        Err(e) => {
            throw(GeneralError::ClassicError(e));
            exit(1)
        }
    };

    println!("Converting level");
    let js: mc_classic_js::Data = match convert::classic_to_js(classic, 1, 1) {
        Ok(c) => c,
        Err(e) => {
            throw(GeneralError::ConversionError(e));
            exit(1)
        }
    };

    println!("Serializing level");
    let serialized: [String; 2] = mc_classic_js::serialize_data(js);

    println!("Writing level");

    match config.output_settings.output_mode {
        0 => {
             _ = match mc_classic_js::write_data(config.output_settings.output_folder, serialized, config.output_settings.output_website)  {
                Ok(c) => c,
                Err(e) => {
                    throw(GeneralError::WriteError(e));
                    exit(1)
                }
            };
        },
        1 => {
           _ = mc_classic_js::write_local_storage_command(
            config.output_settings.output_folder + "/" + &config.output_settings.output_file,
            serialized)
        }
        _ => {
            throw(GeneralError::InvalidMode(config.output_settings.output_mode));
            exit(1);
        }
    }

    println!("Press Enter to Exit");
    let mut s: String = String::from("");
    std::io::stdin().read_line(&mut s).expect("");
    return;

}

fn build_settings () -> Result<(),GeneralError>{
    let mut file = OpenOptions::new()
    .append(true)
    .create(true)
    .open("config.toml").unwrap();

    file.write("[input-settings]\n".as_bytes())?;
    file.write(format!(r#"input-folder = "{INPUT_FOLDER}""#).as_bytes())?;
    file.write("\n".as_bytes())?;
    file.write(format!(r#"input-file = "{INPUT_FILE}""#).as_bytes())?;
    file.write("\n\n".as_bytes())?;
    file.write("[output-settings]\n".as_bytes())?;
    file.write(format!(r#"output-mode = {OUTPUT_MODE}"#).as_bytes())?;
    file.write("\n".as_bytes())?;
    file.write(format!(r#"output-folder = "{OUTPUT_FOLDER}""#).as_bytes())?;
    file.write("\n".as_bytes())?;
    file.write(format!(r#"output-file = "{OUTPUT_FILE}""#).as_bytes())?;
    file.write("\n".as_bytes())?;
    file.write(format!(r#"output-website = "{OUTPUT_WEBSITE}""#).as_bytes())?;
    return Ok(())
}

fn throw (e: GeneralError) {
    eprintln!("Error: {:#?}", e);
    println!("Press Enter to Exit");
    let mut s: String = String::from("");
    std::io::stdin().read_line(&mut s).expect("");
    std::process::exit(1)
}