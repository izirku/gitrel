# GitRel

> GitHub binary manager - install and update single binary releases via GitHub API

## Install/Update

via *Curl/Shell* script (*recommended*):

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/izirku/gitrel/main/xtra/install.sh)"
```

via Cargo:

```bash
cargo install gitrel
```

Linux and macOS users may want to update *gitrel* just like any other managed binary,
if installed via *Curl/Shell* script. Simply add *gitrel* itself,
as one of the installed packages:

```bash
gitrel install izirku/gitrel
```

## Usage

If a `repo` has the same name as `user`/`org`, a *short-hand* can be used,
so, "`gitrel install rust-analyzer`" is the same as
"`gitrel install https://github.com/rust-analyzer/rust-analyzer@*`".
Where "`@*`" stands for a *latest release*.

A *SEMVER*, matching a release tag can be specified as `[repo/]user@SEMVER`.

When updating a binary, `gitrel`, if applicable, will first try to update to
a newer compatible semantic version. It will also check the remote's
*release tag* publish date to what is installed locally. If a remote has a newer
publish date, `gitrel` will download and install it. This is useful for
installing and keeping up to date some *rolling* releases,
such as `rust-analyzer@nightly`.

Glob pattern specified by `--asset-glob` only matches against an asset file name
and its extension. Therefore use of `**` and `/` is disallowed here. Glob pattern
specified by `--entry-glob` on the other hand, matches against a full path inside
of an archive, and use of `**` and `/` is possible there.

### Basic Install Examples

```bash
# install binary (specific tag)
gitrel install rust-analyzer@nightly

# install binary (latest release)
gitrel install gokcehan/lf

# install binary (match tag to a SemVer)
gitrel install https://github.com/JohnnyMorganz/StyLua@^0.11
```

### Advanced Install Examples

Since there is no single standard on naming release artifacts,
automatic matching algorithm may fail. This is why a manual matching
escape hatch is provided. We can use RegEx and glob patterns, to match
against asset names and archive entires. Here are some examples:

```bash
# force install binary, rename, use glob pattern asset match
gitrel install -fa "bbl-v*_osx" -r bbl cloudfoundry/bosh-bootloader

# install binary, strip, use RegEx pattern asset match
gitrel install -sA "^yq_darwin_amd64$" mikefarah/yq

# install binary, strip, use glob pattern match on asset and archive entry
gitrel install -sa "staticcheck_darwin_amd64.tar.gz" \
  -e "**/staticcheck" -r staticcheck dominikh/go-tools
```

Sometimes there is a need to run a command after binary has been installed.
For example, `michaeleisel/zld` (a faster alternative to `ld` on macOS) is
dynamically linked against full *XCode*, and fails to run for users with
*CommandLine Tools* only.

It's possible to fix this by running a command post install (currently Linux/macOS only):

```bash
# note that env variable `$f` containing installed binary path is exported
gitrel install -fsa "zld.zip" \
  -p "/usr/local/bin" \
  -x 'install_name_tool -add_rpath /Library/Developer/CommandLineTools/usr/lib $f' \
  michaeleisel/zld

# or use ":bin:" which gets substituted as well
gitrel install -fsa "zld.zip" \
  -p "/usr/local/bin" \
  -x "install_name_tool -add_rpath /Library/Developer/CommandLineTools/usr/lib :bin:" \
  michaeleisel/zld
```

### Update, Uninstall, Info, and List Examples

Running `update` will honor any manual matching, renames, binary strip (Linux/macOS),
and *post install command* to run (currently Linux/macOS only), as they were specified
during the `install`. Subsequently, `update` command may fail if a never binary version
uses a sufficiently different packaging schema. In such case, force re-install such binary
(i.e. `gitrel install -f ...`), providing new pattern matching parameters.

```bash
# update all installed binaries
gitrel update

# update a single binary
gitrel update bbl

# uninstall binaries
gitrel uninstall bbl yq

# get information about a release on GitHub
gitrel info izirku/gitrel

# list installed binaries
gitrel list

# list installed binaries, displaying installation path
gitrel list -w
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
> ***Use at your own risk***.
