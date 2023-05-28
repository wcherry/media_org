use audiotags::Tag;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::Error;
use std::path::PathBuf;

pub struct Params {
    pub copy: bool,
    pub metadata: bool,
    pub recursive: bool,
}

struct Info {
    artist: String,
    album: String,
    track: String,
    song: String,
    ext: Option<String>,
}

pub fn process_dir(input_dir: PathBuf, output_dir: PathBuf, args: &Params) -> Result<(), Error> {
    eprintln!("Processing files in directory {}", input_dir.display());

    let paths = fs::read_dir(input_dir).unwrap();
    let filename_regex = Regex::new(r"([^-]*)-([^-]*)-(\d+) (.*)\.(flac|mp3)").unwrap();

    let mut directories: HashMap<PathBuf, bool> = HashMap::new();

    for path in paths {
        let path = path?;
        let filename = path.file_name();
        let copy = args.copy;
        let metadata = args.metadata;
        let recursive = args.recursive;

        if path.metadata().unwrap().is_dir() {
            if recursive {
                process_dir(path.path(), output_dir.clone(), args)?;
            } else {
                println!("Skipping directory {}", path.path().display())
            }
        } else {
            let filename = filename.to_str().unwrap();
            let info: Option<Info> = if metadata {
                println!("Working {}", path.path().display());
                let mut ext = None;
                if filename.ends_with(".mp3") {
                    ext = Some(String::from("mp3"));
                }
                if filename.ends_with(".flac") {
                    ext = Some(String::from("flac"));
                }
                if ext.is_none() {
                    eprintln!(
                        "Extension not supported for file {} ",
                        path.path().display()
                    );
                    continue;
                }
                let tag = Tag::new().read_from_path(path.path()).unwrap();
                let artist = String::from(tag.artist().unwrap_or(""));
                let album = String::from(tag.album_title().unwrap_or(""));
                let track = format!("{}", tag.track_number().unwrap_or(0));
                let song = String::from(tag.title().unwrap_or(""));
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
                    let ext = Some(String::from(captures.get(5).map_or("", |m| m.as_str())));
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
                if ext.is_none() {
                    eprintln!("Extension not found for file {} ", path.path().display());
                    continue;
                }
                let ext = ext.unwrap();
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

                if copy {
                    let result = fs::copy(&path.path(), &dir);
                    if result.is_err() {
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
