//! `xx version`: the version of xx.

pub fn run() {
    println!("xx {}", env!("CARGO_PKG_VERSION"));
}
