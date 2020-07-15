use std::{fs, io, path};
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
    let start = path::PathBuf::from(".");
    let root = find_yarn_lock(start)?;
    let cwd = get_working_dir(root, args.dir);

    match script {
        Some(s) => println!("yarn run --cwd={} {}", cwd.display(), s),
        None => println!("cd {}", cwd.display()),
    }

    Ok(())
}

fn get_working_dir(root: path::PathBuf, dir: Option<path::PathBuf>) -> path::PathBuf {
    match dir {
        Some(d) => root.join(d),
        None => path::PathBuf::from("."),
    }
}

fn find_yarn_lock(current: path::PathBuf) -> Result<path::PathBuf, io::Error> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let file_name = entry.file_name();

        if file_name == "yarn.lock" {
            return Ok(current);
        }
    }

    let next = current.join(path::PathBuf::from(".."));

    if next == path::PathBuf::from("/") {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not find a yarn.lock",
        ));
    }

    find_yarn_lock(next)
}
