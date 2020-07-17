use glob::glob;
use std::{env, fs, io, path};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "mr",
    about = "Easy monorepo context switching and script running."
)]
struct Cli {
    #[structopt(short, long, takes_value = false, help = "Lists package paths")]
    list: bool,

    #[structopt(
        parse(from_os_str),
        required_unless = "list",
        help = "Directory to change into"
    )]
    dir: Option<path::PathBuf>,

    #[structopt(help = "Yarn script to run")]
    script: Option<String>,
}

fn main() -> Result<(), io::Error> {
    let args = Cli::from_args();
    let list = args.list;

    if list {
        print!("echo \"{}\"", list_package_directories()?);
        return Ok(());
    }

    let script = args.script;
    let dir = args.dir.expect("Directory argument is required");
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

fn find_package_directories(
    root: &path::PathBuf,
) -> Result<Vec<path::PathBuf>, Box<dyn std::error::Error>> {
    let package_json = String::from_utf8(fs::read(root.join("package.json"))?)?;
    let patterns = &json::parse(&package_json)?["workspaces"]["packages"];
    let mut workspaces = Vec::new();

    for pattern in patterns.members() {
        let full_pattern = match pattern.as_str() {
            Some(p) => root.join(p),
            None => root.to_path_buf(),
        };
        let pattern_str =
            match full_pattern.to_str() {
                Some(p) => p,
                None => return Err(Box::new(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "There may be an invalid unicode character in your workspace package patterns",
                ))),
            };

        for dir in glob(pattern_str)? {
            workspaces.push(dir?);
        }
    }

    Ok(workspaces)
}

fn list_package_directories() -> io::Result<String> {
    let mut result = String::new();
    let root = find_root_dir(&env::current_dir()?)?;
    let packages = match find_package_directories(&root) {
        Ok(pkgs) => pkgs,
        Err(_e) => Vec::new(),
    };

    for pkg in packages {
        result.push_str(pkg.to_str().unwrap_or(""));
        result.push_str("\n");
    }

    Ok(result)
}
