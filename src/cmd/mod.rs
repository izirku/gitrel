mod info;
mod install;
mod list;

pub use self::info::info;
pub use self::install::install;
pub use self::list::list;

// pub mod list;
// pub mod update;

// pub struct App<'a> {
//     client: Option<Client>,
//     temp_dir: Option<TempDir>,
//     arg_matches: &'a ArgMatches,
// }

// impl<'a> App<'a> {
//     pub fn light(arg_matches: &'a ArgMatches) -> Self {
//         App {
//             client: None,
//             temp_dir: None,
//             arg_matches,
//         }
//     }

//     pub fn full(arg_matches: &'a ArgMatches) -> Self {
//         App {
//             client: Some(reqwest::Client::new()),
//             temp_dir: Some(tempfile::tempdir().expect("creating a temp dir failed")),
//             arg_matches,
//         }
//     }
    
//     pub fn info(&self) {
//         self::info::process(cm, matches)
//     }
// }
