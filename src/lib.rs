use std::slice;

type Args = [String];

// TODO: figure out how to remove duplication or simplify forwarding to Cmd
pub struct App<'data> {
    args: Vec<String>,
    data: Data<'data>,
}

impl<'data> App<'data> {
    pub fn new(name: &'data str) -> Self {
        Self::new_args(name, std::env::args())
    }

    // TODO: is there an IntoIter?
    pub fn new_args(name: &'data str, args: impl Iterator<Item = String>) -> Self {
        let args = args.skip(1).collect();

        // TODO: probably want a new() for command
        Self {
            args,
            data: Data {
                name,
                matched: true,
                ..Data::default()
            },
        }
    }

    // TODO: what if there's not an action associated with the App?
    pub fn run<F>(self, run: F)
    where
        F: FnOnce(&Args),
    {
        let mut run_help = false;

        let mut app = self.subcmd(|sub| sub.name("help").run(|_| run_help = true));

        app.data.run(app.args.as_slice(), run);

        if run_help {
            println!("{:#?}", app.data);
        }
    }

    pub fn subcmd<F>(mut self, sub: F) -> Self
    where
        F: for<'a> FnOnce(NewCmd<'a>) -> Data<'data>,
    {
        self.data.subcmd(&self.args, sub);
        self
    }
}

// TODO: more specific name
// TODO: this lifetime is kind of awkward
#[derive(Default, Debug)]
pub struct Data<'a> {
    name: &'a str,
    description: Option<&'a str>,
    sub: Vec<Data<'a>>,
    subtree_ran: bool,

    // TODO: is there a better way to do this?
    matched: bool,
}

impl<'a> Data<'a> {
    pub fn run<F>(&mut self, args: &Args, run: F)
    where
        F: FnOnce(&Args),
    {
        if self.matched && !self.subtree_ran {
            eprintln!("{} running", self.name);
            (run)(args);
            self.subtree_ran = true;
        } else {
            eprintln!("{} not running", self.name);
        }
    }

    pub fn subcmd<F>(&mut self, args: &Args, sub: F)
    where
        F: for<'b> FnOnce(NewCmd<'b>) -> Data<'a>,
    {
        let cmd = NewCmd { args: args.iter() };
        let data = (sub)(cmd);

        self.subtree_ran |= data.subtree_ran;
        self.sub.push(data);
    }
}

// TODO: rename this, probably should just be Cmd, and Cmd should be something else
pub struct NewCmd<'args> {
    args: slice::Iter<'args, String>,
}

impl<'args> NewCmd<'args> {
    pub fn name(mut self, name: &str) -> Cmd<'args, '_> {
        let matched = matches!(self.args.next(), Some(next) if next == name);

        Cmd {
            args: self.args,
            data: Data {
                name,
                matched,
                ..Data::default()
            },
        }
    }
}

pub struct Cmd<'args, 'data> {
    args: slice::Iter<'args, String>,
    data: Data<'data>,
}

impl<'args, 'data> Cmd<'args, 'data> {
    pub fn run<F>(mut self, run: F) -> Data<'data>
    where
        F: FnOnce(&Args),
    {
        self.data.run(self.args.as_slice(), run);
        self.data
    }

    pub fn subcmd<F>(mut self, sub: F) -> Self
    where
        F: for<'a> FnOnce(NewCmd<'a>) -> Data<'data>,
    {
        // TODO: this clone is cheap, right?
        self.data.subcmd(self.args.as_slice(), sub);

        self
    }

    pub fn flag(mut self, name: &str, value: &mut u32) -> Self {
        match self.args.as_slice() {
            [first, second, ..] => {
                if first == name {
                    match second.parse::<u32>() {
                        Ok(i) => {
                            *value = i;

                            // TODO: do this better
                            self.args.next();
                            self.args.next();

                            self
                        }
                        Err(_) => self,
                    }
                } else {
                    self
                }
            }
            _ => self,
        }
    }

    pub fn description(mut self, description: &'data str) -> Self {
        self.data.description = Some(description);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owned(args: &[&str]) -> Vec<String> {
        args.iter().map(|a| a.to_string()).collect()
    }

    #[test]
    fn no_subcommands_no_flags() {
        let run = |args: Vec<String>| {
            let mut captured: Option<Vec<String>> = None;

            App::new_args("first", args.into_iter()).run(|a| captured = Some(a.to_vec()));

            captured
        };

        let args = vec![
            owned(&["first"]),
            owned(&["first", "one"]),
            owned(&["first", "one", "two"]),
            owned(&["first", "one", "two", "three"]),
            owned(&["first", "one", "two", "three", "four"]),
        ];

        for a in args {
            // TODO: get rid of this clone
            let captured = run(a.clone());

            // If we have a single command and no subcommands or flags, all arguments
            // must be passed to the top-level command.
            let c = captured.expect("command should have run");

            assert_eq!(
                c,
                a[1..],
                "arguments after the name of the command must be passed to run"
            );
        }
    }

    #[test]
    fn one_level_subcommands_no_flags() {
        let run = |args: Vec<String>| {
            eprintln!("args: {args:?}");

            let mut captured_first: Option<Vec<String>> = None;
            let mut captured_second: Option<Vec<String>> = None;
            let mut captured_third: Option<Vec<String>> = None;
            let mut captured_fourth: Option<Vec<String>> = None;

            App::new_args("first", args.into_iter())
                .subcmd(|cmd| {
                    cmd.name("second")
                        .run(|a| captured_second = Some(a.to_vec()))
                })
                .subcmd(|cmd| cmd.name("third").run(|a| captured_third = Some(a.to_vec())))
                .subcmd(|cmd| {
                    cmd.name("fourth")
                        .run(|a| captured_fourth = Some(a.to_vec()))
                })
                .run(|a| captured_first = Some(a.to_vec()));

            (
                captured_first,
                captured_second,
                captured_third,
                captured_fourth,
            )
        };

        let args = vec![
            owned(&["first", "second", "one"]),
            owned(&["first", "second", "one", "two"]),
            owned(&["first", "second", "one", "two", "three"]),
            owned(&["first", "second", "one", "two", "three", "four"]),
        ];

        for a in args {
            let (captured_first, captured_second, captured_third, captured_fourth) = run(a.clone());

            assert!(
                captured_first.is_none(),
                "should not have run the outer command"
            );

            assert!(
                captured_third.is_none(),
                "should not have run the third command"
            );

            assert!(
                captured_fourth.is_none(),
                "should not have run the fourth command"
            );

            let c = captured_second.expect("subcommand should have run");
            assert_eq!(
                c,
                &a[2..],
                "arguments after the name of the subcommand must be passed to the subcommand"
            );
        }

        let args = vec![
            owned(&["first", "third", "one"]),
            owned(&["first", "third", "one", "two"]),
            owned(&["first", "third", "one", "two", "three"]),
            owned(&["first", "third", "one", "two", "three", "four"]),
        ];

        for a in args {
            let (captured_first, captured_second, captured_third, captured_fourth) = run(a.clone());

            assert!(
                captured_first.is_none(),
                "should not have run the outer command"
            );

            assert!(
                captured_second.is_none(),
                "should not have run the third command"
            );

            assert!(
                captured_fourth.is_none(),
                "should not have run the fourth command"
            );

            let c = captured_third.expect("subcommand should have run");
            assert_eq!(
                c,
                &a[2..],
                "arguments after the name of the subcommand must be passed to the subcommand"
            );
        }

        let args = vec![
            owned(&["first", "fourth", "one"]),
            owned(&["first", "fourth", "one", "two"]),
            owned(&["first", "fourth", "one", "two", "three"]),
            owned(&["first", "fourth", "one", "two", "three", "four"]),
        ];

        for a in args {
            let (captured_first, captured_second, captured_third, captured_fourth) = run(a.clone());

            assert!(
                captured_first.is_none(),
                "should not have run the outer command"
            );

            assert!(
                captured_second.is_none(),
                "should not have run the third command"
            );

            assert!(
                captured_third.is_none(),
                "should not have run the third command"
            );

            let c = captured_fourth.expect("subcommand should have run");
            assert_eq!(
                c,
                &a[2..],
                "arguments after the name of the subcommand must be passed to the subcommand"
            );
        }

        let args = vec![
            owned(&["first"]),
            owned(&["first", "one"]),
            owned(&["first", "one", "two"]),
            owned(&["first", "one", "two", "three"]),
            owned(&["first", "one", "two", "three", "four"]),
        ];

        for a in args {
            let (captured_first, captured_second, captured_third, captured_fourth) = run(a.clone());

            let c = captured_first.expect("first command should have run");
            assert_eq!(
                c,
                a[1..],
                "all arguments should have been passed to first command"
            );

            assert!(
                captured_second.is_none(),
                "should not have run the second command"
            );

            assert!(
                captured_third.is_none(),
                "should not have run the third command"
            );

            assert!(
                captured_fourth.is_none(),
                "should not have run the fourth command"
            );
        }
    }

    #[test]
    fn two_level_subcommand_no_flags() {
        let run = |args: Vec<String>| {
            let mut captured_first: Option<Vec<String>> = None;
            let mut captured_second: Option<Vec<String>> = None;
            let mut captured_third: Option<Vec<String>> = None;
            let mut captured_fourth: Option<Vec<String>> = None;

            App::new_args("first", args.into_iter())
                .subcmd(|cmd| {
                    cmd.name("second")
                        .subcmd(|cmd| cmd.name("third").run(|a| captured_third = Some(a.to_vec())))
                        .subcmd(|cmd| {
                            cmd.name("fourth")
                                .run(|a| captured_fourth = Some(a.to_vec()))
                        })
                        .run(|a| captured_second = Some(a.to_vec()))
                })
                .run(|a| captured_first = Some(a.to_vec()));

            (
                captured_first,
                captured_second,
                captured_third,
                captured_fourth,
            )
        };

        let args = vec![
            owned(&["first", "second", "third", "one"]),
            owned(&["first", "second", "third", "one", "two"]),
            owned(&["first", "second", "third", "one", "two", "three"]),
            owned(&["first", "second", "third", "one", "two", "three", "four"]),
        ];

        for a in args {
            eprintln!("testing with {:?}", a);
            let (captured_first, captured_second, captured_third, captured_fourth) = run(a.clone());

            assert!(
                captured_first.is_none(),
                "should not have run the outer command"
            );
            assert!(
                captured_second.is_none(),
                "should not have run the first subcommand"
            );
            assert!(
                captured_fourth.is_none(),
                "should not have run the fourth subcommand"
            );
            let c = captured_third.expect("subcommand should have run");
            assert_eq!(
                c,
                &a[3..],
                "arguments after the name of the subcommand must be passed to the subcommand"
            );
        }

        let args = vec![
            owned(&["first", "second", "fourth", "one"]),
            owned(&["first", "second", "fourth", "one", "two"]),
            owned(&["first", "second", "fourth", "one", "two", "three"]),
            owned(&["first", "second", "fourth", "one", "two", "three", "four"]),
        ];

        for a in args {
            let (captured_first, captured_second, captured_third, captured_fourth) = run(a.clone());

            assert!(
                captured_first.is_none(),
                "should not have run the outer command"
            );
            assert!(
                captured_second.is_none(),
                "should not have run the first subcommand"
            );
            assert!(
                captured_third.is_none(),
                "should not have run the third subcommand"
            );
            let c = captured_fourth.expect("subcommand should have run");
            assert_eq!(
                c,
                &a[3..],
                "arguments after the name of the subcommand must be passed to the subcommand"
            );
        }

        let args = vec![
            owned(&["first", "second", "one"]),
            owned(&["first", "second", "one", "two"]),
            owned(&["first", "second", "one", "two", "three"]),
            owned(&["first", "second", "one", "two", "three", "four"]),
        ];

        for a in args {
            let (captured_first, captured_second, captured_third, captured_fourth) = run(a.clone());

            assert!(
                captured_first.is_none(),
                "should not have run the outer command"
            );
            assert!(
                captured_third.is_none(),
                "should not have run the innermost subcommand"
            );
            assert!(
                captured_fourth.is_none(),
                "should not have run the innermost subcommand"
            );
            let c = captured_second.expect("subcommand should have run");
            assert_eq!(
                c,
                &a[2..],
                "arguments after the name of the subcommand must be passed to the subcommand"
            );
        }

        let args = vec![
            owned(&["first"]),
            owned(&["first", "one"]),
            owned(&["first", "one", "two"]),
            owned(&["first", "one", "two", "three"]),
            owned(&["first", "one", "two", "three", "four"]),
        ];

        for a in args {
            let (captured_first, captured_second, captured_third, captured_fourth) = run(a.clone());

            assert!(
                captured_second.is_none(),
                "should not have run the subcommand"
            );
            assert!(
                captured_third.is_none(),
                "should not have run the innermost subcommand"
            );
            assert!(
                captured_fourth.is_none(),
                "should not have run the fourth subcommand"
            );

            let c = captured_first.expect("first command should have run");
            assert_eq!(
                c,
                a[1..],
                "all arguments should have been passed to first command"
            );
        }
    }
}
