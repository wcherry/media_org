use audiotags::Tag;
use clap::Parser;
use regex::Regex;
use std::any::Any;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::iter::Map;
use std::path::PathBuf;
use std::{fs::File, io::Error};
// use std::io::{BufWriter, Write};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, arg_required_else_help(true))]
struct Args {
    /// Schema to extract
    #[arg(short, long, required = false)]
    dir: Option<String>,

    /// Database url to connect to
    #[arg(short, long, required = false)]
    out: Option<String>,

    /// Database url to connect to
    #[arg(short, long, required = false)]
    copy: bool,

    /// Enable Metadata refresh
    #[arg(short, long, required = false)]
    metadata: bool,
}

struct Info {
    artist: String,
    album: String,
    track: String,
    song: String,
    ext: String,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let input_dir = if let Some(dir) = args.dir {
        PathBuf::from(dir).canonicalize()?
    } else {
        env::current_dir()?.canonicalize()?
    };

    let output_dir = if let Some(dir) = args.out {
        PathBuf::from(dir)
    } else {
        env::current_dir()?.canonicalize()?
    };

    eprintln!("Processing files in directory {}", input_dir.display());

    let paths = fs::read_dir(input_dir).unwrap();
    let filename_regex = Regex::new(r"([^-]*)-([^-]*)-(\d+) (.*)\.(flac|mp3)").unwrap();

    let mut directories: HashMap<PathBuf, bool> = HashMap::new();

    for path in paths {
        let path = path?;
        let filename = path.file_name();

        if path.metadata().unwrap().is_dir() {
            println!("Skipping directory {}", path.path().display())
        } else {
            let filename = filename.to_str().unwrap();
            let info: Option<Info> = if args.metadata {
                println!("Working {}", path.path().display());
                let tag = Tag::new().read_from_path(path.path()).unwrap();
                let artist = String::from(tag.artist().unwrap_or(""));
                let album = String::from(tag.album_title().unwrap_or(""));
                let track = format!("{}", tag.track_number().unwrap_or(0));
                let song = String::from(tag.title().unwrap_or(""));
                let ext = String::from(if filename.ends_with(".mp3") {
                    "mp3"
                } else {
                    "flac"
                });
                Some(Info {
                    artist,
                    album,
                    track,
                    song,
                    ext,
                })
            } else {
                let captures = filename_regex.captures(filename);
                if let Some(captures) = captures {
                    let artist = String::from(captures.get(1).map_or("", |m| m.as_str()));
                    let album = String::from(captures.get(2).map_or("", |m| m.as_str()));
                    let track = String::from(captures.get(3).map_or("", |m| m.as_str()));
                    let song = String::from(captures.get(4).map_or("", |m| m.as_str()));
                    let ext = String::from(captures.get(5).map_or("", |m| m.as_str()));
                    Some(Info {
                        artist,
                        album,
                        track,
                        song,
                        ext,
                    })
                } else {
                    eprintln!("Pattern not mached for file {} ", path.path().display());
                    None
                }
            };
            if let Some(info) = info {
                let (artist, album, track, song, ext) =
                    (info.artist, info.album, info.track, info.song, info.ext);
                println!("Filename : {}", filename);
                println!("Artist   : {}", artist);
                println!("Album    : {}", album);
                println!("Track    : {}", track);
                println!("Song     : {}", song);
                println!("Extension: {}\n", ext);

                let mut dir = PathBuf::from(&output_dir);
                dir.push(&artist);
                if !directories.contains_key(&dir) {
                    //mkdir dir
                    if !dir.exists() {
                        eprintln!("Make dir {}", dir.display());
                        fs::create_dir(&dir)?;
                    }
                    directories.insert(dir, true);
                }
                let mut dir = PathBuf::from(&output_dir);
                dir.push(&artist);
                dir.push(&album);
                if !directories.contains_key(&dir) {
                    //mkdir dir
                    if !dir.exists() {
                        eprintln!("Make dir {}", dir.display());
                        fs::create_dir(&dir)?;
                    }
                    directories.insert(dir.clone(), true);
                }
                dir.push(format!("{track} {song}.{ext}"));

                if args.copy {
                    let result = fs::copy(&path.path(), &dir);
                    if (result.is_err()) {
                        eprintln!(
                            "Error copying file {} to {}",
                            path.path().display(),
                            dir.display()
                        );
                    }
                } else {
                    fs::rename(&path.path(), &dir)?;
                }
            }
        }
    }

    return Ok(());
}
