mod manager;
mod package;

pub use self::manager::ConfigurationManager;
pub use self::package::{Package, PackageMap, PackageMatchKind};
