pub fn purple(msg: &str) {
    println!("\x1b[48;5;57m{}\x1b[0m", msg);
}

pub fn pink(msg: &str) {
    println!("\x1b[48;5;132m{}\x1b[0m", msg);
}
