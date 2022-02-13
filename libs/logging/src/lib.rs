// use std::{fs::{self, OpenOptions}, path::PathBuf};
// use std::io::prelude::*;

pub fn purple(msg: &str) {
    println!("\x1b[48;5;57m{}\x1b[0m", msg);
}

pub fn pink(msg: &str) {
    println!("\x1b[48;5;132m{}\x1b[0m", msg);
}

// TODO: Figure out if I want to log to files or not

// #[cfg(debug_assertions)]
// #[cfg(feature = "logging")]
pub fn log(snippet: String) {
    println!("{}", snippet);
}

// #[cfg(not(debug_assertions))]
// #[cfg(feature = "logging")]
// pub fn log(snippet: String) {
//     if !PathBuf::from("/tmp/tendril-log").exists() {

//     std::thread::spawn(move || fs::write("/tmp/tendril-log", snippet).unwrap())
//         .join()
//         .unwrap();
//     } else {
//         let mut file = OpenOptions::new()
//             .write(true)
//             .append(true)
//             .open("/tmp/tendril-log")
//             .unwrap();

//         std::thread::spawn(move || writeln!(file, "\n{}", snippet).unwrap())
//             .join()
//             .unwrap();
//     }
// }
