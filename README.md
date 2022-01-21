# GitRel

> Install and update binaries via GitHub Releases API

## Usage Tips

If a `repo` has the same name as `user`/`org`, a *short-hand* can be used,
so, "`gitrel install rust-analyzer`" is the same as
"`gitrel install https://github.com/rust-analyzer/rust-analyzer@*`".
Where "`@*`" stands for a *latest release*, and isn't parsed as a
*semantic version*.

When updating a binary, `gitrel`, if applicable, will first try to update to
a newer compatible semantic version. It will also check the remote's
*release tag* publish date to what is installed locally. If a remote has a newer
publish date, `gitrel` will download and install it. This is useful for
installing and keeping up to date some *rolling* releases,
such as `rust-analyzer@nightly`.

### Examples

```bash
# install a package (specific tag)
gitrel install rust-analyzer@nightly

# install a package (latest release)
gitrel install rust-analyzer

# install a package (match tag to a SemVer)
gitrel install https://github.com/JohnnyMorganz/StyLua@^0.11

# install a package, strip executable, use asset name RegEx filter to resolve conflicts
gitrel install -s -A "^yq_darwin_amd64$" mikefarah/yq

# update all installed packages
gitrel update
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
