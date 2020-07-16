# `mr`

For easy navigation and script running within a monorepo. Stands for "monorepo", pronounced "mister".

## Usage

```
$ pwd
~/code/some-monorepo

$ mr -l
~/code/some-monorepo/packages/pkg-a
~/code/some-monorepo/packages/pkg-b

$ mr packages/pkg-a test
yarn run v1.19.1
$ jest
42/42 tests passing
✨  Done in 13.37s.

$ pwd
~/code/some-monorepo

$ mr packages/pkg-a

$ pwd
~/code/some-monorepo/packages/pkg-a

$ mr pkg-b

$ pwd
~/code/some-monorepo/packages/pkg-b

$ mr -
~/code/some-monorepo/packages/pkg-a

$ mr . test
yarn run v1.19.1
$ jest
42/42 tests passing
✨  Done in 13.37s.

$ mr /

$ pwd
~/code/some-monorepo
```

## Prerequisites

- shell (`cd` is used to change directories)
- yarn

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