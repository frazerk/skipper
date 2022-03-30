use clap::{arg, command, Command};

fn main() {
    let cmd = |name: &str| {
        Command::new(name)
            .about("bench test")
            .trailing_var_arg(true)
            .arg(arg!([input] ...))
    };

    let inner = |name: &str| {
        cmd(name)
            .subcommand(cmd("a"))
            .subcommand(cmd("b"))
            .subcommand(cmd("c"))
            .subcommand(cmd("d"))
            .subcommand(cmd("e"))
            .subcommand(cmd("f"))
            .subcommand(cmd("g"))
            .subcommand(cmd("h"))
            .subcommand(cmd("i"))
            .subcommand(cmd("j"))
    };

    let matches = command!()
        .subcommand(inner("one"))
        .subcommand(inner("two"))
        .subcommand(inner("three"))
        .subcommand(inner("four"))
        .subcommand(inner("five"))
        .subcommand(inner("six"))
        .subcommand(inner("seven"))
        .subcommand(inner("eight"))
        .subcommand(inner("nine"))
        .subcommand(inner("ten"))
        .subcommand(inner("eleven"))
        .subcommand(inner("twelve"))
        .subcommand(inner("thirteen"))
        .subcommand(inner("fourteen"))
        .subcommand(inner("fifteen"))
        .subcommand(inner("sixteen"))
        .subcommand(inner("seventeen"))
        .subcommand(inner("eighteen"))
        .subcommand(inner("nineteen"))
        .subcommand(inner("twenty"))
        .trailing_var_arg(true)
        .arg(arg!([input] ...))
        .get_matches();

    match matches.subcommand() {
        Some((first, first_args)) => match first_args.subcommand() {
            Some((second, second_args)) => {
                print(second, second_args);
            }
            None => {
                print(first, first_args);
            }
        },
        None => print("clap", &matches),
    };
}

fn print(name: &str, args: &clap::ArgMatches) {
    println!(
        "{name} {:?}",
        args.values_of("input")
            .unwrap_or_default()
            .collect::<Vec<&str>>()
    )
}
