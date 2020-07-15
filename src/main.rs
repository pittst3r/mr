use std::{env, fs, io, path};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short, long, parse(from_os_str))]
    dir: Option<path::PathBuf>,

    #[structopt()]
    script: Option<String>,
}

fn main() -> Result<(), io::Error> {
    let args = Cli::from_args();
    let script = args.script;
    let root = &mut env::current_dir()?;

    find_yarn_lock(root)?;

    let cwd = match args.dir {
        Some(d) => root.join(d).canonicalize()?,
        None => env::current_dir()?,
    };

    match script {
        Some(s) => println!("yarn run --cwd={} {}", cwd.display(), s),
        None => println!("cd {}", cwd.display()),
    }

    Ok(())
}

fn find_yarn_lock(current: &mut path::PathBuf) -> Result<(), io::Error> {
    for entry in fs::read_dir(&mut *current)? {
        let entry = entry?;
        let file_name = entry.file_name();

        if file_name == "yarn.lock" {
            return Ok(());
        }
    }

    current.push("..");

    if *current == path::PathBuf::from("/") {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not find a yarn.lock",
        ));
    }

    find_yarn_lock(current)
}
