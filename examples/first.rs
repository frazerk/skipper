use skipper::*;

fn main() {
    let mut a = 1;

    App::new("first")
        .subcmd("second", |second| {
            second
                .subcmd("third", |third| {
                    third.run(|i| println!("3 {}", i.join("-")))
                })
                .run(|i| {
                    a += 1;
                    println!("2 {} {}", i.join("/"), a)
                })
        })
        .subcmd("fourth", |fourth| {
            let mut flag1 = 0;
            let mut flag2 = 0;
            fourth
                .flag("-t", &mut flag1)
                .flag("-u", &mut flag2)
                .run(|i| println!("4 {} FLAG1: {}, FLAG2: {}", i.join("^"), flag1, flag2))
        })
        .run(first);
}

fn first(args: &[String]) {
    println!("1 {}", args.join("+"));
}
