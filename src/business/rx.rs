use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    // source: https://regex101.com/r/Ly7O1x/1070
    pub static ref SEMVER: Regex = Regex::new(r"(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)(?:-(?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?").expect("error parsing regex");

// regex crate doesnt support branch reset...
//     pub static ref REPO_SPEC: Regex = Regex::new(r"^(?:(\w[\w-]*)|(?:(\w[\w-]*)@(.*))|(?:(\w[\w-]*)/(\w[\w-]*))|(?:(\w[\w-]*)/(\w[\w-]*)@(.*)))$").expect("error parsing regex");
//     pub static ref REPO_SPEC: RegexSet = RegexSet::new(&[
//         r"^(?P<repo>\w[\w-]*)$",
//         r"^(?P<repo>\w[\w-]*)@(?P<tag>.*)$",
//         r"^(?P<user>\w[\w-]*)/(?P<repo>\w[\w-]*)$",
//         r"^(?P<user>\w[\w-]*)/(?P<repo>\w[\w-]*)@(?P<tag>.*)$",
//         ]).expect("error parsing regex");
}
