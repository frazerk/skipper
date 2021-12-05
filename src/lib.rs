type Args = [String];

// TODO: figure out how to remove duplication or simplify forwarding to Cmd
pub struct App<'a> {
    cmd: Cmd<'a>,
}

impl<'a> App<'a> {
    pub fn new(name: &'a str) -> Self {
        Self::new_args(name, std::env::args())
    }

    // TODO: is there an IntoIter?
    pub fn new_args(name: &'a str, args: impl Iterator<Item = String>) -> Self {
        let args = args.skip(1).collect();

        // TODO: probably want a new() for command
        Self {
            cmd: Cmd {
                args: Some(args),
                data: Data {
                    name,
                    ..Data::default()
                },
            },
        }
    }

    // TODO: what if there's not an action associated with the App?
    pub fn run<F>(self, run: F)
    where
        F: FnOnce(&Args),
    {
        let mut run_help = false;

        let app = self.subcmd("help", |help| help.run(|_| run_help = true));
        let data = app.cmd.run(run);

        if run_help {
            println!("{:#?}", data);
        }
    }

    pub fn subcmd<F>(mut self, name: &'a str, sub: F) -> Self
    where
        F: FnOnce(Cmd<'a>) -> Data<'a>,
    {
        self.cmd = self.cmd.subcmd(name, sub);
        self
    }
}

// TODO: more specific name
#[derive(Default, Debug)]
pub struct Data<'a> {
    name: &'a str,
    description: Option<&'a str>,
    sub: Vec<Data<'a>>,

    // TODO: is there a better way to do this?
    // TODO: also implement detecting if commands actually ran
    executed: bool,
}

pub struct Cmd<'a> {
    args: Option<Vec<String>>,
    data: Data<'a>,
}

impl<'a> Cmd<'a> {
    pub fn run<F>(self, run: F) -> Data<'a>
    where
        F: FnOnce(&Args),
    {
        if let Some(args) = self.args {
            (run)(args.as_slice());
        }
        self.data
    }

    pub fn subcmd<F>(mut self, name: &'a str, sub: F) -> Self
    where
        F: FnOnce(Cmd<'a>) -> Data<'a>,
    {
        let data = Data {
            name,
            ..Data::default()
        };

        let cmd = match self.args.as_deref() {
            Some([first, ..]) if first == name => {
                // TODO: find a more graceful way to do this
                let args = self.args.take().map(|mut v| {
                    v.remove(0);
                    v
                });
                Cmd { args, data }
            }
            _ => Cmd { args: None, data },
        };

        let data = (sub)(cmd);

        self.data.sub.push(data);

        self
    }

    pub fn flag(mut self, name: &str, value: &mut u32) -> Self {
        match self.args.as_deref() {
            Some([first, second, ..]) => {
                if first == name {
                    match second.parse::<u32>() {
                        Ok(i) => {
                            // TODO: do this better
                            self.args = self.args.map(|mut v| {
                                v.remove(0);
                                v.remove(0);
                                v
                            });
                            *value = i;
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

    pub fn description(mut self, description: &'a str) -> Self {
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
            let mut captured_first: Option<Vec<String>> = None;
            let mut captured_second: Option<Vec<String>> = None;
            let mut captured_third: Option<Vec<String>> = None;
            let mut captured_fourth: Option<Vec<String>> = None;

            App::new_args("first", args.into_iter())
                .subcmd("second", |second| {
                    second.run(|a| captured_second = Some(a.to_vec()))
                })
                .subcmd("third", |third| {
                    third.run(|a| captured_third = Some(a.to_vec()))
                })
                .subcmd("fourth", |fourth| {
                    fourth.run(|a| captured_fourth = Some(a.to_vec()))
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
                .subcmd("second", |second| {
                    second
                        .subcmd("third", |third| {
                            third.run(|a| captured_third = Some(a.to_vec()))
                        })
                        .subcmd("fourth", |fourth| {
                            fourth.run(|a| captured_fourth = Some(a.to_vec()))
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
