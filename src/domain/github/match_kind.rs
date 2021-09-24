#[derive(Debug)]
pub enum MatchKind {
    Exact,
    Latest,
    SemVer,
}

pub fn match_kind(str: &str) -> MatchKind {
    if str == "*" {
        MatchKind::Latest
    } else if semver::VersionReq::parse(str).is_ok() {
        MatchKind::SemVer
    } else {
        MatchKind::Exact
    }
}
