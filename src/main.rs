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
    let start = env::current_dir()?;
    let root = find_root_dir(&start)?;
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

fn find_root_dir(current: &path::PathBuf) -> Result<path::PathBuf, io::Error> {
    if current.canonicalize()? == path::PathBuf::from("/") {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not find a yarn.lock",
        ));
    }

    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let file_name = entry.file_name();

        if file_name == "yarn.lock" {
            return Ok(current.canonicalize()?);
        }
    }

    let next = current.join("..");

    find_root_dir(&next)
}
