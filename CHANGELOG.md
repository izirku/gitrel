# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## \[0.2.10] - 2022-02-21

### Added

*   Created a workflow to publish to crates.io
*   No changes otherwise

## \[0.2.9] - 2022-02-20

### Added (Linux/macOS only)

*   Execute a command after installation with `gitrel install -x CMD`
    *   use either `$f` (must be escaped inside double quotes, i.e. `"echo \$f"`)\
        or `:bin:` in `CMD`, to substitute for a full path of installed binary
    *   subsequent `update` invocations will re-run such commands

## \[0.2.8] - 2022-02-20

### Added

*   allow installation path override with `gitrel install -p <path>`
*   `gitrel list -w`, in addtion, will list installation paths

### Fixed

*   Windows build supporting above *additions*
