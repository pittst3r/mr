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
    let dir = args.dir.unwrap_or(env::current_dir()?);
    let cwd = compute_cwd(dir)?;

    match script {
        Some(s) => println!("yarn run --cwd={} {}", cwd.display(), s),
        None => println!("cd {}", cwd.display()),
    }

    Ok(())
}

fn compute_cwd(dir: path::PathBuf) -> io::Result<path::PathBuf> {
    let start = env::current_dir()?;
    let root = find_root_dir(&start)?;

    if dir == path::PathBuf::from("/") {
        return Ok(root);
    }

    package_path(root, start, &dir)?.canonicalize()
}

fn package_path(
    root: path::PathBuf,
    base: path::PathBuf,
    pattern: &path::PathBuf,
) -> io::Result<path::PathBuf> {
    let full_path = base.join(pattern);

    if full_path.exists() {
        return Ok(full_path);
    }

    if root == base {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not find given directory within project",
        ));
    }

    package_path(root, base.join(path::PathBuf::from("..")), pattern)
}

fn find_root_dir(current: &path::PathBuf) -> io::Result<path::PathBuf> {
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
