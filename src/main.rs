use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::DirBuilder;
use std::fs::ReadDir;
use std::io::Error;
use std::path::{Path, PathBuf};

struct Config {
    pub verbose: bool,
    pub recursive: bool,
    pub path: Option<PathBuf>,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.is_empty() {
        help();
        return;
    }

    let mut config = Config {
        verbose: false,
        recursive: false,
        path: None,
    };

    for (index, arg) in args.iter().enumerate() {
        if index == 0 {
            continue;
        } else if arg.eq("--help") {
            help();
            return;
        } else if arg.eq("--verbose") {
            config.verbose = true;
        } else if arg.eq("--recursive") {
            config.recursive = true;
        } else {
            if let Some(path) = config.path {
                panic!("You've specified two paths, {} and {}", &path.display(), &arg);
            }
            config.path = Some(Path::new(&arg).to_path_buf());
        }
    }

    let mut file_db = FileDatabase::new(config);
    file_db.scan();
    file_db.check_duplicates();
}

fn help() {}

struct FileDatabase {
    config: Config,
    database: HashMap<u16, Vec<PathBuf>>,
}

impl FileDatabase {
    fn new(config: Config) -> Self {
        Self {
            config,
            database: HashMap::new(),
        }
    }

    fn scan(&mut self) {
        if let Some(path) = self.config.path.to_owned() {
            self.add_folder_to_database(&path);
        } else {
            help();
            panic!("No path to scan specified, exiting...")
        }
    }

    fn check_duplicates(&self) {
        let mut duplicates_found = false;
        for (hash, paths) in &self.database {
            if paths.len() > 1 {
                duplicates_found = true;
                println!("There are duplicate files:");
                for path in paths {
                    println!("    {}", path.display());
                }
            }
        }

        if !duplicates_found {
            println!("No duplicates found.");
        }
    }

    fn add_folder_to_database(&mut self, path: &PathBuf) {
        if self.config.verbose {
            println!("Scanning directory `{}`", path.display());
        }

        let read_dir = match fs::read_dir(path) {
            Ok(read_dir) => read_dir,
            Err(e) => {
                println!("Unable to read dir `{}` because {}", path.display(), e);
                return;
            }
        };

        for child in read_dir {
            let child = match child {
                Ok(child) => child,
                Err(e) => {
                    println!("Something wrong with the child idk `{}`", e);
                    return;
                }
            };

            if child.path().is_dir() {
                if self.config.recursive {
                    self.add_folder_to_database(&child.path().as_path().to_path_buf());
                }
                continue;
            }

            let content = match fs::read(child.path()) {
                Ok(content) => content,
                Err(e) => {
                    println!("Unable to read contents of {:?} for reason {}", child.path(), e);
                    return;
                }
            };

            if content.is_empty() {
                if self.config.verbose {
                    println!("Skipping empty file {}", child.path().display());
                }
                continue;
            }

            let hash = crc::crc16::checksum_usb(content.as_slice());

            match self.database.get_mut(&hash) {
                None => {
                    let mut vec = Vec::new();
                    vec.push(child.path());
                    self.database.insert(hash, vec);
                }
                Some(vec) => vec.push(child.path()),
            }
        }
    }
}