use std::{env, fs, io, path};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    dir: path::PathBuf,

    #[structopt()]
    script: Option<String>,
}

fn main() -> Result<(), io::Error> {
    let args = Cli::from_args();
    let script = args.script;
    let dir = args.dir;
    let cwd = compute_cwd(dir)?;

    match script {
        Some(s) => print!("yarn run --cwd={} {}", cwd.display(), s),
        None => print!("cd {}", cwd.display()),
    }

    Ok(())
}

fn compute_cwd(dir: path::PathBuf) -> io::Result<path::PathBuf> {
    if dir == path::PathBuf::from("-") {
        return Ok(dir);
    }

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
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let file_name = entry.file_name();

        if file_name == "yarn.lock" {
            return current.canonicalize();
        }
    }

    let next = current.join("..");

    if next.canonicalize()? == path::PathBuf::from("/") {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not find a yarn.lock",
        ));
    }

    find_root_dir(&next)
}
