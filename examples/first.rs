use skipper::*;

fn main() {
    let mut a = 1;

    App::new("first")
        .run(first)
        .sub2("second", |cmd| {
            cmd.run(|i| {
                a += 1;
                println!("2 {} {}", i.join("/"), a)
            })
            .subcmd(Cmd::new("third").run(|i| println!("3 {}", i.join("-"))))
        })
        .sub2("fourth", |cmd| {
            let mut flag = 0;
            cmd.flag("-t", &mut flag)
                .run(move |i| println!("4 {} FLAG: {}", i.join("^"), flag))
        })
        .execute();
}

fn first(args: &[String]) {
    println!("1 {}", args.join("+"));
}
