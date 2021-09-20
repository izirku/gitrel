use crate::business::conf::ConfigurationManager;
use crate::error::AppError;
use crate::foundation::util::svec2_col_maj_max_lens_unchecked;
use crate::Result;

/// List requested packages
pub fn process(cm: &ConfigurationManager) -> Result<()> {
    let packages = match cm.get_packages() {
        Ok(packages) => packages,
        Err(AppError::NotFound) => {
            println!("nothing is installed on this system");
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    let mut cols = Vec::with_capacity(packages.len());

    for (name, pkg_spec) in packages.into_iter() {
        let repo = format!("[https://github.com/{}]", &pkg_spec.repo);
        cols.push(vec![name, pkg_spec.requested, repo]);
    }

    let max_lens = svec2_col_maj_max_lens_unchecked(&cols);

    println!(
        "{:<w_name$} {:<w_ver$} REPO\n",
        "BIN",
        "TAG",
        w_name = max_lens[0],
        w_ver = max_lens[1],
    );

    for row in &cols {
        if let [name, ver, repo] = &row[..] {
            println!(
                "{:<w_name$} {:<w_ver$} {}",
                name,
                ver,
                repo,
                w_name = max_lens[0],
                w_ver = max_lens[1],
            );
        }
    }
    Ok(())
}
