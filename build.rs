use std::env::var;

fn main() {
    let _profile = var("PROFILE").unwrap();
}
