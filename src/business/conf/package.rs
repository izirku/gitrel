use super::InstalledPackage;
use super::RequestedPackage;
#[derive(Debug)]
pub struct Package<'a> {
    pub name: &'a str,
    pub requested: Option<&'a RequestedPackage>,
    pub installed: Option<&'a InstalledPackage>,
}

impl<'a> Package<'a> {
    pub fn repo(&self) -> Option<&'a str> {
        match (self.requested, self.installed) {
            (Some(requested), _) => Some(requested.repo.as_str()),
            (_, Some(_installed)) => unimplemented!(),
            _ => None,
        }
    }
}
