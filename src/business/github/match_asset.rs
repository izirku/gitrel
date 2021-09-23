use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

lazy_static! {
    // source: https://regex101.com/r/Ly7O1x/1070
    static ref TERMS: Regex = Regex::new(r"(x86_64|x86\-64|[a-zA-Z0-9]+)").expect("error parsing regex");
}

pub fn satisfied(str: &str) -> bool {
    for term in TERMS.find_iter(str) {
        if EXCLUDE_SET.contains(term.as_str()) {
            return false;
        }
    }
    true
}
