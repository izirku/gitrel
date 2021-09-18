use super::RequestedSpec;

pub struct Package<'a> {
    pub bin: &'a str,
    pub requested: &'a RequestedSpec,
}
