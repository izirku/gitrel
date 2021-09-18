use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageMatchKind {
    /// Release tag follows a well behaved semver
    SemVer,
    /// compare a date of a release tag update and what we have installed
    Date,
    /// including pre-releases (i.e. "nightly" tag commonly set to be a pre-release)
    Named,
    /// Latest non pre-release
    Latest,
    /// Release tag matches a certain regex
    RegEx,
}

impl Default for PackageMatchKind {
    fn default() -> Self {
        Self::SemVer
    }
}
