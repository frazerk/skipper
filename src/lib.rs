use std::env;

type Args = [String];
type Run<'a> = Box<dyn FnMut(&Args) + 'a>;

pub struct Command<'a> {
    pub name: &'a str,
    pub subcommands: Vec<CommandRun<'a>>,
}

impl<'a> Command<'a> {
    pub fn run<F>(self, f: F) -> CommandRun<'a>
    where
        F: 'a + FnMut(&Args),
    {
        CommandRun {
            command: self,
            run: Box::new(f),
        }
    }
}

pub struct CommandRun<'a> {
    command: Command<'a>,
    run: Box<dyn FnMut(&Args) + 'a>,
}

impl<'a> CommandRun<'a> {
    pub fn execute(&mut self) {
        let args: Vec<String> = env::args().skip(1).collect();
        self.execute_args(&args);
    }

    fn execute_args(&mut self, args: &Args) {
        match self.find(args) {
            Some((r, a)) => (r)(a),
            None => println!("ERROR: no command found"),
        }
    }

    fn find<'b>(&mut self, args: &'b Args) -> Option<(&mut Run<'a>, &'b [String])> {
        if args.is_empty() {
            return Some((&mut self.run, args));
        }

        let next_sub = &args[0];

        let found = self
            .command
            .subcommands
            .iter_mut()
            .find(|sc| sc.command.name == next_sub);

        match found {
            Some(sc) => sc.find(&args[1..]),
            None => Some((&mut self.run, args)),
        }
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
        let run = |args: &Vec<String>| {
            let mut captured: Option<Vec<String>> = None;

            let mut cmd = Command {
                name: "first",
                subcommands: vec![],
            }
            .run(|a| captured = Some(a.to_vec()));
            cmd.execute_args(args);
            drop(cmd);

            captured
        };

        let args = vec![
            owned(&[]),
            owned(&["one"]),
            owned(&["one", "two"]),
            owned(&["one", "two", "three"]),
            owned(&["one", "two", "three", "four"]),
        ];

        for a in args {
            let captured = run(&a);

            // If we have a single command and no subcommands or flags, all arguments
            // must be passed to the top-level command.
            assert_eq!(captured, Some(a));
        }
    }

    #[test]
    fn one_level_subcommand_no_flags() {
        let run = |args: &Vec<String>| {
            let mut captured_first: Option<Vec<String>> = None;
            let mut captured_second: Option<Vec<String>> = None;

            let mut cmd = Command {
                name: "first",
                subcommands: vec![Command {
                    name: "second",
                    subcommands: vec![],
                }
                .run(|a| captured_second = Some(a.to_vec()))],
            }
            .run(|a| captured_first = Some(a.to_vec()));
            cmd.execute_args(args);
            drop(cmd);

            (captured_first, captured_second)
        };

        let args = vec![
            owned(&["second", "one"]),
            owned(&["second", "one", "two"]),
            owned(&["second", "one", "two", "three"]),
            owned(&["second", "one", "two", "three", "four"]),
        ];

        for a in args {
            let (captured_first, captured_second) = run(&a);

            assert!(
                captured_first.is_none(),
                "should not have run the outer command"
            );
            let c = captured_second.expect("subcommand should have run");
            assert_eq!(
                c,
                &a[1..],
                "arguments after the name of the subcommand must be passed to the subcommand"
            );
        }

        let args = vec![
            owned(&[]),
            owned(&["one"]),
            owned(&["one", "two"]),
            owned(&["one", "two", "three"]),
            owned(&["one", "two", "three", "four"]),
        ];

        for a in args {
            let (captured_first, captured_second) = run(&a);

            assert!(
                captured_second.is_none(),
                "should not have run the subcommand"
            );
            assert_eq!(
                captured_first,
                Some(a),
                "all arguments should have been passed to first command"
            );
        }
    }

    #[test]
    fn two_level_subcommand_no_flags() {
        let run = |args: &Vec<String>| {
            let mut captured_first: Option<Vec<String>> = None;
            let mut captured_second: Option<Vec<String>> = None;
            let mut captured_third: Option<Vec<String>> = None;

            let mut cmd = Command {
                name: "first",
                subcommands: vec![Command {
                    name: "second",
                    subcommands: vec![Command {
                        name: "third",
                        subcommands: vec![],
                    }
                    .run(|a| captured_third = Some(a.to_vec()))],
                }
                .run(|a| captured_second = Some(a.to_vec()))],
            }
            .run(|a| captured_first = Some(a.to_vec()));

            cmd.execute_args(args);
            drop(cmd);

            (captured_first, captured_second, captured_third)
        };

        let args = vec![
            owned(&["second", "third", "one"]),
            owned(&["second", "third", "one", "two"]),
            owned(&["second", "third", "one", "two", "three"]),
            owned(&["second", "third", "one", "two", "three", "four"]),
        ];

        for a in args {
            let (captured_first, captured_second, captured_third) = run(&a);

            assert!(
                captured_first.is_none(),
                "should not have run the outer command"
            );
            assert!(
                captured_second.is_none(),
                "should not have run the first subcommand"
            );
            let c = captured_third.expect("subcommand should have run");
            assert_eq!(
                c,
                &a[2..],
                "arguments after the name of the subcommand must be passed to the subcommand"
            );
        }

        let args = vec![
            owned(&["second", "one"]),
            owned(&["second", "one", "two"]),
            owned(&["second", "one", "two", "three"]),
            owned(&["second", "one", "two", "three", "four"]),
        ];

        for a in args {
            let (captured_first, captured_second, captured_third) = run(&a);

            assert!(
                captured_first.is_none(),
                "should not have run the outer command"
            );
            assert!(
                captured_third.is_none(),
                "should not have run the innermost subcommand"
            );
            let c = captured_second.expect("subcommand should have run");
            assert_eq!(
                c,
                &a[1..],
                "arguments after the name of the subcommand must be passed to the subcommand"
            );
        }

        let args = vec![
            owned(&[]),
            owned(&["one"]),
            owned(&["one", "two"]),
            owned(&["one", "two", "three"]),
            owned(&["one", "two", "three", "four"]),
        ];

        for a in args {
            let (captured_first, captured_second, captured_third) = run(&a);

            assert!(
                captured_second.is_none(),
                "should not have run the subcommand"
            );
            assert!(
                captured_third.is_none(),
                "should not have run the innermost subcommand"
            );
            assert_eq!(
                captured_first,
                Some(a),
                "all arguments should have been passed to first command"
            );
        }
    }
}
