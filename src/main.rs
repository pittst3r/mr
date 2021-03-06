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

struct Mr {
    root: path::PathBuf,
    base: path::PathBuf,
}

impl Mr {
    fn new() -> Mr {
        let start = env::current_dir().unwrap_or(path::PathBuf::from("."));
        let root = find_root_dir(&start).unwrap_or(path::PathBuf::from("."));

        Mr { root, base: start }
    }

    fn cwd(self, pattern: path::PathBuf) -> io::Result<path::PathBuf> {
        if pattern == path::PathBuf::from("-") {
            return match env::var("OLDPWD") {
                Ok(prev) => Ok(path::PathBuf::from(prev)),
                Err(_e) => Ok(env::current_dir().unwrap_or(path::PathBuf::from("."))),
            };
        }

        if pattern == path::PathBuf::from("/") {
            return Ok(self.root);
        }

        if pattern == path::PathBuf::from(".") {
            return Ok(self.base);
        }

        self.package_path(pattern)?.canonicalize()
    }

    fn list_package_directories(self) -> io::Result<String> {
        let mut result = String::new();
        let packages = match self.find_package_directories() {
            Ok(pkgs) => pkgs,
            Err(_e) => Vec::new(),
        };

        for pkg in packages {
            result.push_str(pkg.to_str().unwrap_or(""));
            result.push_str("\n");
        }

        Ok(result)
    }

    fn package_path(self, pattern: path::PathBuf) -> io::Result<path::PathBuf> {
        let packages = match self.find_package_directories() {
            Ok(pkgs) => pkgs,
            Err(_e) => Vec::new(),
        };

        let valid_packages = packages
            .iter()
            .filter(|pkg| pkg.to_str().unwrap_or("") != "");

        let pat_str = match pattern.to_str() {
            Some(pat) => pat,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Could not parse given directory pattern",
                ))
            }
        };

        let mut matches = valid_packages.filter(|pkg| {
            let pkg_str = match pkg.to_str() {
                Some(p) => p,
                None => return false,
            };

            if pkg_str.ends_with(pat_str) {
                return true;
            }

            return false;
        });

        match matches.next() {
            Some(m) => Ok(path::PathBuf::from(m)),
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Could not find given directory within project",
            )),
        }
    }

    fn find_package_directories(self) -> Result<Vec<path::PathBuf>, Box<dyn std::error::Error>> {
        let package_json = String::from_utf8(fs::read(self.root.join("package.json"))?)?;
        let patterns = &json::parse(&package_json)?["workspaces"]["packages"];
        let mut workspaces = Vec::new();

        for pattern in patterns.members() {
            let full_pattern = match pattern.as_str() {
                Some(p) => self.root.join(p),
                None => self.root.to_path_buf(),
            };
            let pattern_str = match full_pattern.to_str() {
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

fn main() -> Result<(), io::Error> {
    let args = Cli::from_args();
    let list = args.list;
    let mr = Mr::new();

    if list {
        print!("echo \"{}\"", mr.list_package_directories()?);
        return Ok(());
    }

    let pattern = args.dir.unwrap_or(path::PathBuf::from("."));
    let cwd = mr.cwd(pattern)?;

    match args.script {
        Some(s) => print!("yarn run --cwd={} {}", cwd.display(), s),
        None => print!("cd {}", cwd.display()),
    }

    Ok(())
}
