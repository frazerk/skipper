use std::env;

fn main() {
    println!(
        "baseline {:?}",
        env::args().skip(1).collect::<Vec<String>>()
    );
}
