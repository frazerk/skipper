use skipper::*;

fn main() {
    App::new("first")
        .subcmd(second)
        .subcmd(|cmd| {
            let mut flag1 = 0;
            let mut flag2 = 0;
            cmd.name("fourth")
                .description("what's this? it must be the fourth subcommand")
                .flag("-t", &mut flag1)
                .flag("-u", &mut flag2)
                .run(|i| println!("4 {} FLAG1: {}, FLAG2: {}", i.join("^"), flag1, flag2))
        })
        .run(first);
}

fn first(args: &Args) {
    println!("1 {}", args.join("+"));
}

fn second(cmd: NewCmd) -> Data<'static> {
    let mut a = 1;
    cmd.name("second")
        .description("hey, it's the second subcommand")
        .subcmd(|cmd| {
            cmd.name("third")
                .description("oh dang dude this is the third subcommand")
                .run(|i| println!("3 {}", i.join("-")))
        })
        .run(|i| {
            a += 1;
            println!("2 {} {}", i.join("/"), a)
        })
}
