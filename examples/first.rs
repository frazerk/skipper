use skipper::*;

fn main() {
    App::new("first")
        .run(first)
        .subcmd(second())
        .subcmd(
            Cmd::new("fourth")
                .description("what's this? it must be the fourth subcommand")
                .run(|i| println!("4 {}", i.join("^"))),
        )
        .exec();
}

fn first(args: &Args) {
    println!("1 {}", args.join("+"));
}

fn second() -> Cmd<'static> {
    let mut a = 1;
    Cmd::new("second")
        .description("hey, it's the second subcommand")
        .run(move |i| {
            a += 1;
            println!("2 {} {}", i.join("/"), a)
        })
        .subcmd(
            Cmd::new("third")
                .description("oh dang dude this is the third subcommand")
                .run(|i| println!("3 {}", i.join("-"))),
        )
}
