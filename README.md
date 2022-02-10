![GitHub Workflow Status](https://img.shields.io/github/workflow/status/izirku/gitrel/CI)
[![license](http://img.shields.io/badge/license-Apache%20v2-blue.svg)](https://raw.githubusercontent.com/izirku/gitrel/master/LICENSE-APACHE)
[![license](http://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/izirku/gitrel/master/LICENSE-MIT)
[![Buy me a coffee](https://img.shields.io/badge/buy%20me%20a%20coffee-donate-yellow.svg)](https://ko-fi.com/izirku)

# GitRel

> Install and update binaries via GitHub Releases API

<p align="center"><img src="/xtra/gitrel_demo.gif?raw=true"/></p>

## Install/Update

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/izirku/gitrel/main/xtra/install.sh)"
```

## Usage

If a `repo` has the same name as `user`/`org`, a *short-hand* can be used,
so, "`gitrel install rust-analyzer`" is the same as
"`gitrel install https://github.com/rust-analyzer/rust-analyzer@*`".
Where "`@*`" stands for a *latest release*.

A _SEMVER_, matching a release tag can be specified as `[repo/]user@SEMVER`.

When updating a binary, `gitrel`, if applicable, will first try to update to
a newer compatible semantic version. It will also check the remote's
*release tag* publish date to what is installed locally. If a remote has a newer
publish date, `gitrel` will download and install it. This is useful for
installing and keeping up to date some *rolling* releases,
such as `rust-analyzer@nightly`.

Glob pattern specified by `--asset-glob` only matches against an asset file name and its extension. Therefore use of `**` and `/` is disallowed here. Glob pattern specified by `--entry-glob` on the other hand, matches agains a full path inside of an archive, and use of `**` and `/` is possible there.

### Examples

```bash
# install a package (specific tag)
gitrel install rust-analyzer@nightly

# install a package (latest release)
gitrel install gokcehan/lf

# install a package (match tag to a SemVer)
gitrel install https://github.com/JohnnyMorganz/StyLua@^0.11

# force install a package, rename final binary, use glob pattern asset match
gitrel install -fa "bbl-v*_osx" -r bbl cloudfoundry/bosh-bootloader

# install a package, strip executable, use RegEx pattern asset match
gitrel install -sA "^yq_darwin_amd64$" mikefarah/yq

# update all installed packages
gitrel update

# update a single package
gitrel update bbl

# uninstall packages
gitrel uninstall bbl yq
```

*NOTE*: Regardless of OS kind, binary files are "installed" under `~/.local/bin`
or `~/bin` directory, if it exists. Otherwise, `~/.local/bin` directory is
created, and binaries are placed there.

## Configuration

Configuration files are stored in `~/.config/gitrel` directory, regardless of
an operating system kind. Currently, it only stores the `packages.json` there.

# Disclamer

> Author and contributors bear no responsibilities whatsoever for any issues
> caused by the use of this software, or software installed via this software.
> __*Use at your own risk*__.
