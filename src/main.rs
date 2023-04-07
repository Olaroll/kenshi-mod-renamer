use std::{env, io};
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io::BufRead;
use std::path::PathBuf;

use anyhow::{Context, Result};

const MOD_EXTENSION: &str = "mod";

struct RenameEntry {
    from: PathBuf,
    to: OsString,
}

impl Display for RenameEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} -> {:?}", self.from.file_name().unwrap(), self.to)
    }
}

fn main() -> Result<()> {
    let res = inner_main()
        .context("Error");

    if let Err(err) = res {
        eprintln!("{:#}", err);
    }

    eprint!("Press enter to close...");
    io::stdin().read_line(&mut String::new())?;

    Ok(())
}

fn inner_main() -> Result<()> {
    let cwd = env::current_dir()
        .context("couldn't get current working directory")?;

    let read_dir = fs::read_dir(&cwd)
        .context("couldn't read contents of current directory")?;

    let rename_list: Vec<_> = read_dir
        .filter_map(|file| {
            // Get all dirs in current directory
            let entry = file.ok()?;
            entry.metadata().ok()?.is_dir().then(|| entry.path())
        })
        .filter_map(|mod_dir| {
            // Find .mod file in each dir
            // If not found, then skip
            let mod_file = fs::read_dir(&mod_dir).ok()?
                .filter_map(|res| res.ok())
                .map(|entry| entry.path())
                .find(|file_path|
                    file_path.extension().map(|ext| ext == MOD_EXTENSION).unwrap_or(false)
                )?;

            Some(RenameEntry {
                from: mod_dir,
                to: mod_file.file_stem()?.into(),
            })
        })
        .filter(|entry| {
            // Skip ones with the correct name already
            entry.from.file_stem().unwrap() != entry.to
        })
        .collect();

    if rename_list.is_empty() {
        eprintln!("Couldn't find any paths to rename");
        eprintln!("This message can also appear when everything already has the correct name");
        return Ok(());
    }

    rename_list.iter()
        .for_each(|entry| println!("{}", entry));

    eprint!("Do you wish to rename the above {} paths? (y/n) ", rename_list.len());

    match io::stdin().lock().lines().next() {
        Some(Ok(response)) if response.to_ascii_lowercase() == "y" => {}
        _ => {
            eprintln!("Operation canceled");
            return Ok(());
        }
    }

    let mut success_count = 0;
    for entry in rename_list {
        let from = entry.from;
        let to = from.parent().unwrap().join(entry.to);

        match fs::rename(&from, &to) {
            Ok(_) => success_count += 1,
            Err(err) => {
                eprintln!("Failed to rename {:?} -> {:?}: {:#}",
                          from.file_name().unwrap(),
                          to.file_name().unwrap(),
                          err,
                )
            }
        }
    }

    eprintln!("Successfully renamed {} dirs", success_count);

    Ok(())
}