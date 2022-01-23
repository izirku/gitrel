#!/bin/bash

OS="$(uname)"
if [[ "${OS}" == "Linux" ]]; then
  URL="https://github.com/izirku/gitrel/releases/latest/download/gitrel-x86_64-unknown-linux-gnu.tar.gz"
  SUBDIR="gitrel-x86_64-unknown-linux-gnu"
elif [[ "${OS}" == "Darwin" ]]; then
  URL="https://github.com/izirku/gitrel/releases/latest/download/gitrel-x86_64-apple-darwin.tar.gz"
  SUBDIR="gitrel-x86_64-apple-darwin"
else
  abort "GitRel installation is currently supported only on macOS and Linux."
fi

curl -sL $URL | tar xz - -C /usr/local/bin --strip-components 1 "${SUBDIR}/gitrel"
chmod +x /usr/local/bin/gitrel
