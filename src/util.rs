#[macro_export]
macro_rules! bench {
    ($name:literal) => {
        fn main() {
            let cmd = |name| {
                Cmd::new(name)
                    .run(move |args| println!("{} {:?}", name, args))
                    .description("bench test")
            };

            let inner = |name| {
                cmd(name)
                    .subcmd(cmd("a"))
                    .subcmd(cmd("b"))
                    .subcmd(cmd("c"))
                    .subcmd(cmd("d"))
                    .subcmd(cmd("e"))
                    .subcmd(cmd("f"))
                    .subcmd(cmd("g"))
                    .subcmd(cmd("h"))
                    .subcmd(cmd("i"))
                    .subcmd(cmd("j"))
            };

            Cmd::root($name)
                .run(|args| println!("{:?}", args))
                .description("bench test")
                .subcmd(inner("one"))
                .subcmd(inner("two"))
                .subcmd(inner("three"))
                .subcmd(inner("four"))
                .subcmd(inner("five"))
                .subcmd(inner("six"))
                .subcmd(inner("seven"))
                .subcmd(inner("eight"))
                .subcmd(inner("nine"))
                .subcmd(inner("ten"))
                .subcmd(inner("eleven"))
                .subcmd(inner("twelve"))
                .subcmd(inner("thirteen"))
                .subcmd(inner("fourteen"))
                .subcmd(inner("fifteen"))
                .subcmd(inner("sixteen"))
                .subcmd(inner("seventeen"))
                .subcmd(inner("eighteen"))
                .subcmd(inner("nineteen"))
                .subcmd(inner("twenty"))
                .execute();
        }
    };
}
