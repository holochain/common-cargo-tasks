#!/bin/sh

die() {
  echo "${@}" 1>&2
  exit 1
}

# make sure git is installed
if ! $(git --version > /dev/null 2>&1); then
  die "please install git"
fi

# make sure cargo is installed
if ! $(cargo --version > /dev/null 2>&1); then
  die "please install rust cargo"
fi

# make sure cargo-task is installed
if ! $(cargo help task > /dev/null 2>&1); then
  if ! $(cargo install cargo-task); then
    die "please install cargo-task"
  fi
fi

# check out the bootstrap file
if ! $(git clone https://github.com/holochain/common-cargo-tasks.git --depth=1 --branch main --single-branch .cargo-task); then
  die "failed to check out the .cargo-task repository"
fi

# execute 'cargo task ops-update'
cargo task ops-update
