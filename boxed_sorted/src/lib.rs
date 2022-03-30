use std::env;

type Args = [String];

#[derive(Default)]
pub struct Cmd<'a> {
    children: Vec<Cmd<'a>>,

    closure: Option<Box<dyn FnOnce(&Args) + 'a>>,

    name: &'a str,
    description: Option<&'a str>,
}

// TODO: split Cmd::root into separate App struct
impl<'a> Cmd<'a> {
    fn build(name: &'a str, _root: bool) -> Self {
        Self {
            name,
            ..Self::default()
        }
    }

    pub fn new(name: &'a str) -> Self {
        Self::build(name, false)
    }

    pub fn root(name: &'a str) -> Self {
        Self::build(name, true)
    }

    pub fn execute(self) {
        let args: Vec<String> = env::args().collect();
        self.execute_with(&args)
    }

    pub fn execute_with(self, args: &Args) {
        // TODO: this explodes if you don't have args
        self.exec(&args[1..]);
    }

    fn exec(mut self, args: &Args) {
        if let [first, a @ ..] = args {
            if let Ok(pos) = self
                .children
                .binary_search_by_key(&first.as_str(), |cmd| cmd.name)
            {
                return self.children.swap_remove(pos).exec(a);
            }
        }

        if let Some(closure) = self.closure {
            closure(args);
        }
    }

    pub fn run(mut self, run: impl FnOnce(&Args) + 'a) -> Cmd<'a> {
        self.closure = Some(Box::new(run));
        self
    }

    // TODO: find shorter name (sub conflicts with common trait)
    pub fn subcmd(mut self, sub: Cmd<'a>) -> Cmd<'a> {
        match self
            .children
            .binary_search_by_key(&sub.name, |cmd| cmd.name)
        {
            Ok(_) => {}
            Err(pos) => self.children.insert(pos, sub),
        }
        self
    }

    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
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
        let run = |args: &[String]| {
            let mut captured: Option<Vec<String>> = None;

            Cmd::root("first")
                .run(|a| captured = Some(a.to_vec()))
                .execute_with(args);

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
            let captured = run(&a);

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
        let run = |args: &[String]| {
            let mut captured_first: Option<Vec<String>> = None;
            let mut captured_second: Option<Vec<String>> = None;
            let mut captured_third: Option<Vec<String>> = None;
            let mut captured_fourth: Option<Vec<String>> = None;

            Cmd::root("first")
                .run(|a| captured_first = Some(a.to_vec()))
                .subcmd(Cmd::new("second").run(|a| captured_second = Some(a.to_vec())))
                .subcmd(Cmd::new("third").run(|a| captured_third = Some(a.to_vec())))
                .subcmd(Cmd::new("fourth").run(|a| captured_fourth = Some(a.to_vec())))
                .execute_with(args);

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
            let (captured_first, captured_second, captured_third, captured_fourth) = run(&a);

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
            let (captured_first, captured_second, captured_third, captured_fourth) = run(&a);

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
            let (captured_first, captured_second, captured_third, captured_fourth) = run(&a);

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
            let (captured_first, captured_second, captured_third, captured_fourth) = run(&a);

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
        let run = |args: &[String]| {
            let mut captured_first: Option<Vec<String>> = None;
            let mut captured_second: Option<Vec<String>> = None;
            let mut captured_third: Option<Vec<String>> = None;
            let mut captured_fourth: Option<Vec<String>> = None;

            Cmd::root("first")
                .run(|a| captured_first = Some(a.to_vec()))
                .subcmd(
                    Cmd::new("second")
                        .run(|a| captured_second = Some(a.to_vec()))
                        .subcmd(Cmd::new("third").run(|a| captured_third = Some(a.to_vec())))
                        .subcmd(Cmd::new("fourth").run(|a| captured_fourth = Some(a.to_vec()))),
                )
                .execute_with(args);

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
            let (captured_first, captured_second, captured_third, captured_fourth) = run(&a);

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
            let (captured_first, captured_second, captured_third, captured_fourth) = run(&a);

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
            let (captured_first, captured_second, captured_third, captured_fourth) = run(&a);

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
            let (captured_first, captured_second, captured_third, captured_fourth) = run(&a);

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
