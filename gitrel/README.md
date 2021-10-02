# GitRel

> Install and manage binaries via GitHub releases

Is under active development, but the primary functionality of downloading
and updating previously installed binaries is functional. An internal registry
format of the installed packages, is a subject to change, but only as
a last resort.

## Usage

If a `repo` has the same name as `user`/`org`, a *short-hand* can be used,
so, "`gitrel install rust-analyzer`" it the same as
"`gitrel install https://github.com/rust-analyzer/rust-analyzer@*`".
Where "`@*`" stands for a *latest release*, and isn't parsed as a
*semantic version*.

When updating a binary, `gitrel`, if applicable, will first try to update to
a newer compatible semantic version. It will also check the remote's
*release tag* publish date to what is installed locally. If a remote has a newer
publish date, `gitrel` will download and insall it. This is usefull for
installing and keeping up to date *rolling* releases,
such as `rust-analyzer@nightly`.

### Examples

```bash
# install a package (specific tag)
gitrel install rust-analyzer@nightly

# install a package (latest release)
gitrel install rust-analyzer

# install a package (match tag to a SemVer)
gitrel install https://github.com/JohnnyMorganz/StyLua@^0.11

# update all installed packages
gitrel update --all
```

*NOTE*: Regardless of OS kind, binary files are "installed" under `~/.local/bin`
or `~/bin` directory, if it exists. Otherwise, `~/.local/bin` directory is
created, and binaries are placed there.

## Configuration

Configuration files are stored in `~/.config/gitrel` directory, regardless of
an operating system kind.

Currently, to use an authenticated access to GitHub, create a  `~/.config/gitrel/github_token.plain` file that contains a *Private Access Token*, or set the
`GITREL_TOKEN` environment variable. This is going to be improved in the future.

## Future Ideas and Improvements

### TO-DO

- [ ] implement `uninstall`
- [ ] implement `install --ensure`
- [ ] change repo layout to use *cargo workspaces*

### Full Version

- Use a proper GitHub App authorization instead of a *personal access token* (PAT)
- GitLab support
- JSON output of installed packages
- Vulnerability scanning if possible

### Lite Version

Create a light version specifically designed to fit well into the automation
pipelines, for DevOps, etc.

- no configuration
- no concurrency
- no temp files (in memory decompression)
- output to a specified or current directory
- smaller list of dependencies
- smaller size as a consequence

# Disclamer

> Author and contributors bear no responsibilites whatsoever for any issues 
> caused by the use of this software, or software installed via this software.
> __*Use at your own risk*__.