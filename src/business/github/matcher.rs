use std::cmp::Ordering;

use anyhow::{Context, Result};
use chrono::NaiveDate;
use regex::Regex;

use crate::business::{
    conf::{Package, PackageMatchKind, RequestedSpec},
    rx,
};

use super::model::Release;

/// Given a GitHub `release` and a `requested` package sped, see if we have a match.
pub fn matches(release: &Release, package: &Package) -> Result<bool> {
    match package.requested {
        // in a simple case, we do a semver match, falling back to an
        // exact match attempt in order to handle something like:
        //   `rust-analyzer = nightly`
        RequestedSpec::Simple(tag) => {
            if let Ok(ver_req) = semver::VersionReq::parse(tag) {
                if let Some(m) = rx::SEMVER.find(&release.tag_name) {
                    let ver_remote = semver::Version::parse(m.as_str())?;
                    return Ok(ver_req.matches(&ver_remote));
                }
            } else {
                if tag == &release.tag_name {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        RequestedSpec::Detailed(details) => {
            let requested;
            // 1. maybe filter
            if let Some(expr) = &details.filter {
                let re = Regex::new(expr).with_context(|| {
                    format!("bad filter RegEx: {}", details.filter.as_ref().unwrap())
                })?;
                if !re.is_match(&release.tag_name) {
                    return Ok(false);
                }
            }
            // 2. maybe extract
            if let Some(expr) = &details.extract {
                let re = Regex::new(expr).with_context(|| {
                    format!("bad filter RegEx: {}", details.filter.as_ref().unwrap())
                })?;
                match re.find(&release.tag_name) {
                    Some(m) => {
                        requested = m.as_str();
                    }
                    None => return Ok(false),
                }
            } else {
                requested = details.matches.as_str();
            }
            // dbg!(requested);

            // 3. try match
            match details.match_kind {
                PackageMatchKind::Named => {
                    if requested == &release.tag_name {
                        return Ok(true);
                    }
                }
                PackageMatchKind::RegEx => {
                    let re = Regex::new(requested)?;
                    if re.is_match(&release.tag_name) {
                        return Ok(true);
                    }
                }
                PackageMatchKind::SemVer => {
                    let ver_req = semver::VersionReq::parse(requested)?;
                    if let Some(m) = rx::SEMVER.find(&release.tag_name) {
                        let ver_remote = semver::Version::parse(m.as_str())?;
                        if ver_req.matches(&ver_remote) {
                            return Ok(true);
                        }
                    }
                }
                PackageMatchKind::Latest => {
                    unimplemented!()
                }
                PackageMatchKind::Date => {
                    // TODO: better OP & date ranges parsing, allow `>=`, `>=DT_START & <=DT_END`
                    let (op, dt_str) = requested.split_at(1);
                    let dt_req =
                        NaiveDate::parse_from_str(dt_str, details.date_fmt.as_ref().unwrap())?;
                    if let Ok(dt_remote) = NaiveDate::parse_from_str(
                        &release.tag_name,
                        details.date_fmt.as_ref().unwrap(),
                    ) {
                        match (op, dt_remote.cmp(&dt_req)) {
                            ("=", Ordering::Equal) => return Ok(true),
                            ("<", Ordering::Less) => return Ok(true),
                            (">", Ordering::Greater) => return Ok(true),
                            _ => return Ok(false),
                        }
                    }
                }
            }
            Ok(false)
        }
    }
}
