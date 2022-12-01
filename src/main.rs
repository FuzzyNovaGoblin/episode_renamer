#![feature(iter_intersperse)]
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};
use std::{env, fs, io::stdin, path::Path, process};
use terminal_size::{terminal_size, Height};

macro_rules! clear_screen {
    () => {{
        if let Some((_, Height(h))) = terminal_size() {
            for _ in 0..h {
                println!();
            }
        }
    }};
}

macro_rules! emsg {
    ($msg:expr) => {
        format!("\n\nthis error had an additional message {}", $msg).as_str()
    };
    ()=>{
        {
            let mut new_lines = String::new();
            if let Some((_, Height(h))) = terminal_size(){
                for _ in 0..h{
                    new_lines.push('\n');
                }
            }
            format!("{new_lines} hey Alex, there was an error that I forgot to account for or something is different between our systems it happened at {}:{} take a screenshot of this or give that location to grant so he can fix it", file!(), line!()).as_str()
        }
    };
}

static SEASON_MATCH_REG: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(".*season.*")
        .case_insensitive(true)
        .build()
        .unwrap()
});
static GOAL_EPISODE_REG: Lazy<Regex> = Lazy::new(|| Regex::new(r"[Ss]\d\d?[Ee]\d\d?").unwrap());
static NUMBERS_OF_EPISODE_REG: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\d+)[^\d]*(\d+)(\..*)").unwrap());

fn season_match_reg() -> &'static Regex {
    &SEASON_MATCH_REG
}
fn goal_episode_reg() -> &'static Regex {
    &GOAL_EPISODE_REG
}
fn numbers_of_episode_reg() -> &'static Regex {
    &NUMBERS_OF_EPISODE_REG
}

fn main() {
    let running_directory = env::current_dir().unwrap();

    clear_screen!();

    println!(
        "program running in {:?} confirm this is correct (type \"yes\")",
        running_directory.as_path()
    );

    loop {
        let mut confirmation = String::new();
        stdin().read_line(&mut confirmation).unwrap();
        let confirmation = confirmation.trim();
        if confirmation.eq_ignore_ascii_case("yes") {
            break;
        } else if confirmation.eq_ignore_ascii_case("quit") {
            return;
        } else {
            println!(
                "you entered {:?}\nenter \"yes\" to confirm or \"quit\" to end the program",
                confirmation
            )
        }
    }

    clear_screen!();

    find_season_directories(&running_directory, &running_directory);
}

fn find_season_directories(start_path: &Path, running_directory: &Path) {
    if let Some(name) = start_path.to_str() {
        if season_match_reg().is_match(name) {
            return handle_season_directory(start_path, running_directory);
        }
    }

    if !start_path.is_dir() {
        return;
    }

    let mut ls_result = match fs::read_dir(start_path) {
        Ok(val) => val
            .filter_map(|res| match res {
                Ok(dir_entry) => Some(dir_entry.path()),
                Err(_) => None,
            })
            .collect::<Vec<_>>(),
        Err(_) => return,
    };
    ls_result.sort();

    for path in ls_result {
        find_season_directories(&path, running_directory)
    }
}

fn handle_season_directory(season_path: &Path, running_directory: &Path) {
    let mut changes = Vec::new();
    for direntry in fs::read_dir(season_path).expect(emsg!()) {
        let path = match direntry {
            Ok(val) => val.path().to_owned(),
            Err(e) => {
                clear_screen!();
                eprintln!("didn't expect this to ever happen can you let me know if you see this at {}:{}\n{e}", file!(), line!());
                continue;
            }
        };

        let file_name = path.file_name().expect(emsg!()).to_str().expect(emsg!());

        if goal_episode_reg().is_match(file_name) {
            continue;
        }

        match numbers_of_episode_reg().captures(file_name) {
            Some(caps) => changes.push((
                String::from(path.to_str().expect(emsg!())),
                Some(format!(
                    "{}/S{:#02}E{:#02}{}",
                    path.parent().expect(emsg!()).to_str().expect(emsg!()),
                    caps.get(1).unwrap().as_str().parse::<u8>().expect(emsg!()),
                    caps.get(2).unwrap().as_str().parse::<u8>().expect(emsg!()),
                    caps.get(3).unwrap().as_str(),
                )),
            )),
            None => changes.push((String::from(path.to_str().expect(emsg!())), None)),
        }
    }
    changes.sort();

    if !changes.is_empty() {
        let failed = changes.iter().filter(|(_, r)| r.is_none());
        let worked = changes
            .iter()
            .filter_map(|(og, new)| new.as_ref().map(|s| (og, s)));
        clear_screen!();
        println!("inside of {:?}\n", season_path);
        if failed.clone().count() > 0 {
            println!("failed to rename the following");
            for n in failed {
                println!("{:?}", n.0);
            }
            print!("\n\n");
        }
        if worked.clone().count() > 0 {
            println!("making the following changes");
            for (og, new) in worked {
                println!(
                    "{:?} -> {:?}",
                    Path::new(og)
                        .strip_prefix(running_directory)
                        .unwrap()
                        .display(),
                    Path::new(new)
                        .strip_prefix(running_directory)
                        .unwrap()
                        .display()
                )
            }

            loop {
                println!("should these changes be made? ([y]es/[n]o/[q]uit)");
                let mut confirmation = String::new();
                stdin().read_line(&mut confirmation).unwrap();
                let confirmation = confirmation
                    .chars()
                    .next()
                    .unwrap_or(' ')
                    .to_ascii_lowercase();
                match confirmation {
                    'y' => {
                        process_name_changes(changes);
                        break;
                    }
                    'n' => break,
                    'q' => process::exit(0),
                    _ => (),
                }
            }
        }
    }
}

fn process_name_changes(changes: Vec<(String, Option<String>)>) {
    for (og, new) in changes {
        if let Some(new_name) = new {
            fs::rename(og, new_name).expect(emsg!());
        }
    }
}
