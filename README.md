# GitRel

> Install and manage binaries via GitHub releases

Is under active development, but the primary functionality of downloading
and updating previously installed binaries is functional. An internal registry
format of the installed packages, is a subject to change, but only as
a last resort.

## Future Ideas and Improvements

### Full Version

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

