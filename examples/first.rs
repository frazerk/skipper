use skipper::*;

fn main() {
    let mut a = 1;

    Cmd::root("first")
        .run(first)
        .subcmd(
            Cmd::new("second")
                .run(|i| {
                    a += 1;
                    println!("2 {} {}", i.join("/"), a)
                })
                .subcmd(Cmd::new("third").run(|i| println!("3 {}", i.join("-")))),
        )
        .execute();
}

fn first(args: &[String]) {
    println!("1 {}", args.join("+"));
}
