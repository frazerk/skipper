use std::env;

type Args = [String];

pub trait Visitor {
    // TODO: rename
    fn exec(&mut self, args: &Args) -> bool;

    fn execute(&mut self) {
        let args: Vec<String> = env::args().collect();
        self.execute_with(&args)
    }

    fn execute_with(&mut self, args: &Args) {
        self.exec(args);
    }
}

#[derive(Default)]
struct Data<'a> {
    name: &'a str,
    description: Option<&'a str>,

    // TODO: this is ugly as sin
    root: bool,
}

pub struct Cmd<'a, F, N> {
    run: F,
    next: N,
    data: Data<'a>,
}

impl<'a, F, N> Cmd<'a, F, N> {
    pub fn run<G>(self, run: G) -> Cmd<'a, G, N>
    where
        G: FnMut(&Args),
    {
        Cmd {
            run,
            next: self.next,
            data: self.data,
        }
    }

    // TODO: find shorter name (sub conflicts with common trait)
    pub fn subcommand<V: Visitor>(self, sub: V) -> Cmd<'a, F, Chain<V, N>> {
        Cmd {
            next: Chain {
                cur: sub,
                next: self.next,
            },
            run: self.run,
            data: self.data,
        }
    }

    pub fn description(mut self, description: &'a str) -> Self {
        self.data.description = Some(description);
        self
    }

    fn check_run<'b>(&self, args: &'b Args) -> Option<&'b Args> {
        match args {
            [] => None,
            [first, a @ ..] => {
                if first == self.data.name || self.data.root {
                    Some(a)
                } else {
                    None
                }
            }
        }
    }
}

impl<'a, F, N> Visitor for Cmd<'a, F, N>
where
    F: FnMut(&Args), // TODO: can we get this to be FnOnce?
    N: Visitor,
{
    fn exec(&mut self, args: &Args) -> bool {
        match self.check_run(args) {
            Some(args) => {
                if !self.next.exec(args) {
                    (self.run)(args);
                }
                true
            }
            None => false,
        }
    }
}

impl<'a, F> Visitor for Cmd<'a, F, ()>
where
    F: FnMut(&Args), // TODO: can we get this to be FnOnce?,
{
    fn exec(&mut self, args: &Args) -> bool {
        match self.check_run(args) {
            Some(args) => {
                (self.run)(args);
                true
            }
            None => false,
        }
    }
}

impl<'a, N> Visitor for Cmd<'a, (), N>
where
    N: Visitor,
{
    fn exec(&mut self, args: &Args) -> bool {
        match self.check_run(args) {
            Some(args) => {
                self.next.exec(args);
                true
            }
            None => false,
        }
    }
}

impl<'a> Visitor for Cmd<'a, (), ()> {
    fn exec(&mut self, args: &Args) -> bool {
        self.check_run(args).is_some()
    }
}

impl<'a> Cmd<'a, (), ()> {
    pub fn new(name: &'a str) -> Self {
        Self {
            run: (),
            next: (),
            data: Data {
                name,
                ..Data::default()
            },
        }
    }

    pub fn root(name: &'a str) -> Self {
        Self {
            run: (),
            next: (),
            data: Data {
                name,
                root: true,
                ..Data::default()
            },
        }
    }
}

pub struct Chain<C, N>
where
    C: Visitor,
{
    cur: C,
    next: N,
}

impl<C, N> Visitor for Chain<C, N>
where
    C: Visitor,
    N: Visitor,
{
    fn exec(&mut self, args: &Args) -> bool {
        if self.cur.exec(args) {
            true
        } else {
            self.next.exec(args)
        }
    }
}

impl<C> Visitor for Chain<C, ()>
where
    C: Visitor,
{
    fn exec(&mut self, args: &Args) -> bool {
        self.cur.exec(args)
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
                .subcommand(Cmd::new("second").run(|a| captured_second = Some(a.to_vec())))
                .subcommand(Cmd::new("third").run(|a| captured_third = Some(a.to_vec())))
                .subcommand(Cmd::new("fourth").run(|a| captured_fourth = Some(a.to_vec())))
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
                .subcommand(
                    Cmd::new("second")
                        .run(|a| captured_second = Some(a.to_vec()))
                        .subcommand(Cmd::new("third").run(|a| captured_third = Some(a.to_vec())))
                        .subcommand(Cmd::new("fourth").run(|a| captured_fourth = Some(a.to_vec()))),
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
