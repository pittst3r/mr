# `mr`

For easy navigation and script running within a monorepo.

## Installation

Install binary to `$PATH` and put this in your profile:

```sh
function mr() {
  if [ "$1" = "-h" -o "$1" = "--help" ]; then
    mr-util $1
  elif [ -z "$1" ]; then
    mr-util -h
  else
    eval "$(mr-util "$@")"
  fi
}
```

## Usage

Use within a monorepo

```
USAGE:
    mr-util <dir> [script]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <dir>
    <script>
```