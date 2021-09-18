use super::InstalledPackage;
use super::RequestedPackage;
#[derive(Debug)]
pub struct Package<'a> {
    pub name: &'a str,
    pub requested: Option<&'a RequestedPackage>,
    pub installed: Option<&'a InstalledPackage>,
}
