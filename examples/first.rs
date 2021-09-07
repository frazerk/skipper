use skipper::Command;

fn main() {
    let a = 1;

    Command {
        name: "first",
        subcommands: vec![Command {
            name: "second",
            subcommands: vec![],
        }
        .run(|i| println!("2 {} {}", i.join("/"), a))],
    }
    .run(first)
    .execute();
}

fn first(args: &[String]) {
    println!("1 {}", args.join("+"));
}
