pub fn parse_gh_repo_name(str: &str) -> String {
    // TODO: add regex validation here, wrap in Result<_>?
    if str.contains('/') {
        str.to_owned()
    } else {
        format!("{0}/{0}", str)
    }
}

pub fn parse_gh_repo_spec(str: &str) -> (String, String) {
    // TODO: add regex validation here, wrap in Result<_>?
    if str.contains('@') {
        let (name, tag) = str.split_at(str.find('@').unwrap());
        (
            parse_gh_repo_name(name),
            tag.trim_start_matches('@').to_owned(),
        )
    } else {
        (parse_gh_repo_name(str), "*".to_owned())
    }
}
