use super::InstalledPackage;
use super::RequestedPackage;

#[derive(Debug)]
pub struct Package<'a> {
    pub name: &'a str,
    pub match_kind: PackageMatchKind,
    pub requested: Option<&'a RequestedPackage>,
    pub installed: Option<&'a InstalledPackage>,
}

#[derive(Debug)]
pub enum PackageMatchKind {
    Exact,
    Latest,
    SemVer,
    Unknown,
}

impl<'a> Package<'a> {
    pub fn create(
        name: &'a str,
        requested: Option<&'a RequestedPackage>,
        installed: Option<&'a InstalledPackage>,
    ) -> Self {
        let match_kind = if let Some(requested) = requested {
            if requested.version == "*" {
                PackageMatchKind::Latest
            } else if semver::VersionReq::parse(&requested.version).is_ok() {
                PackageMatchKind::SemVer
            } else {
                PackageMatchKind::Exact
            }
        } else {
            PackageMatchKind::Unknown
        };

        Self {
            name,
            match_kind,
            requested,
            installed,
        }
    }

    pub fn repo(&self) -> Option<&'a str> {
        match (self.requested, self.installed) {
            (Some(requested), _) => Some(requested.repo.as_str()),
            (_, Some(_installed)) => unimplemented!(),
            _ => None,
        }
    }
}
