#[macro_use]
extern crate afl;
extern crate serde_luaq;

fn main() {
    fuzz!(|data: &[u8]| {
        let _ = serde_luaq::script(&data, 512);
    });
}
