# TO-DO

- [ ] implement a "packages fix-up" mechanism, where we can track _match patterns_ for some of the
more common and unconventional repo releases
- [ ] implement _gitrel_lite_

## Future Ideas and Improvements

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
