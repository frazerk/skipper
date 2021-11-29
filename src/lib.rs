use std::env;

type Args = [String];

const LEAF: Option<NullCmd> = None;

// TODO: find better name for this
pub trait Exec {
    fn exec(self, args: &Args) -> bool;
}

pub trait Traverse
where
    Self: Sized,
{
    fn traverse<E: Exec>(self, args: &Args, next: Option<E>) -> bool;

    fn sub<S>(self, sub: S) -> SubTraverse<Self, S>
    where
        S: Exec,
    {
        SubTraverse { prev: self, sub }
    }

    fn execute(self) {
        let args: Vec<String> = env::args().skip(1).collect();
        self.execute_with(&args)
    }

    fn execute_with(self, args: &Args) {
        self.traverse(args, LEAF);
    }
}

pub struct App<'a, F>
where
    F: FnOnce(&Args),
{
    name: &'a str,
    run: Option<F>,
}

// TODO: should we have a separate builder?
impl<'a, F> App<'a, F>
where
    F: FnOnce(&Args),
{
    pub fn new(name: &'a str) -> Self {
        Self { name, run: None }
    }

    pub fn run(mut self, run: F) -> Self {
        self.run = Some(run);
        self
    }
}

impl<F> Traverse for App<'_, F>
where
    F: FnOnce(&Args),
{
    fn traverse<E: Exec>(self, args: &Args, next: Option<E>) -> bool {
        println!("APP TRAVERSE: {:?}", args);
        // TODO: remove duplication
        match (args, self.run, next) {
            // If we don't have subcommands or there are no arguments, just execute ourselves.
            (a @ [..], Some(run), None) | (a @ [], Some(run), Some(_)) => {
                println!("RUNNING APP");
                run(a);
            }

            // If we do have subcommands, only execute the root if none of the subcommands
            // matched.
            (a @ [..], Some(run), Some(next)) => {
                if !next.exec(a) {
                    println!("RUNNING APP");
                    run(a);
                }
            }

            // If we do have subcommands, only execute the root if none of the subcommands
            // matched.
            (a @ [..], None, Some(next)) => {
                next.exec(a);
            }

            _ => (),
        }

        true
    }
}

pub struct Cmd<'a, F>
where
    F: FnOnce(&Args),
{
    name: &'a str,
    run: Option<F>,
}

// TODO: duplication with App
impl<'a, F> Cmd<'a, F>
where
    F: FnOnce(&Args),
{
    pub fn new(name: &'a str) -> Self {
        Self { name, run: None }
    }

    pub fn run(mut self, run: F) -> Self {
        self.run = Some(run);
        self
    }

    // TODO: better name
    fn go(self, args: &Args) {
        if let Some(run) = self.run {
            run(args);
        }
    }
}

impl<F> Exec for Cmd<'_, F>
where
    F: FnOnce(&Args),
{
    fn exec(self, args: &Args) -> bool {
        // TODO: remove duplication with traverse
        match args {
            [] => false,
            [first, a @ ..] => {
                if first == self.name {
                    self.go(a);
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl<F> Traverse for Cmd<'_, F>
where
    F: FnOnce(&Args),
{
    fn traverse<E: Exec>(self, args: &Args, next: Option<E>) -> bool {
        println!("COMMAND TRAVERSE: {:?}", args);

        // TODO: remove duplication
        match (args, next) {
            // If we don't have arguments, neither we nor our subcommands should execute.
            ([], _) => false,

            ([first, a @ ..], Some(next)) if first == self.name => {
                if next.exec(a) {
                    true
                } else {
                    self.exec(args)
                }
            }

            ([first, ..], None) if first == self.name => self.exec(args),

            _ => false,
        }
    }
}

pub struct SubTraverse<P, S>
where
    P: Traverse,
    S: Exec,
{
    prev: P,
    sub: S,
}

impl<P, S> Traverse for SubTraverse<P, S>
where
    P: Traverse,
    S: Exec,
{
    fn traverse<E: Exec>(self, args: &Args, next: Option<E>) -> bool {
        println!("SUBTRAVERSE TRAVERSE: {:?}", args);
        self.prev.traverse(
            args,
            Some(SubCommand {
                exec: self.sub,
                next,
            }),
        )
    }
}

impl<P, S> Exec for SubTraverse<P, S>
where
    P: Traverse,
    S: Exec,
{
    fn exec(self, args: &Args) -> bool {
        self.traverse(args, LEAF)
    }
}

pub struct SubCommand<E, S> {
    exec: E,
    next: Option<S>,
}

impl<E, S> Exec for SubCommand<E, S>
where
    E: Exec,
    S: Exec,
{
    fn exec(self, args: &Args) -> bool {
        // TODO: is this order right?
        if self.exec.exec(args) {
            true
        } else if let Some(next) = self.next {
            next.exec(args)
        } else {
            false
        }
    }
}

// TODO: get rid of this
pub struct NullCmd {}

impl Exec for NullCmd {
    fn exec(self, _: &Args) -> bool {
        false
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

            App::new("first")
                .run(|a| captured = Some(a.to_vec()))
                .execute_with(args);

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
            // TODO: get rid of this clone
            let captured = run(&a);

            // If we have a single command and no subcommands or flags, all arguments
            // must be passed to the top-level command.
            assert_eq!(captured, Some(a));
        }
    }

    #[test]
    fn one_level_subcommands_no_flags() {
        let run = |args: &[String]| {
            let mut captured_first: Option<Vec<String>> = None;
            let mut captured_second: Option<Vec<String>> = None;
            let mut captured_third: Option<Vec<String>> = None;
            let mut captured_fourth: Option<Vec<String>> = None;

            App::new("first")
                .run(|a| captured_first = Some(a.to_vec()))
                .sub(Cmd::new("second").run(|a| captured_second = Some(a.to_vec())))
                .sub(Cmd::new("third").run(|a| captured_third = Some(a.to_vec())))
                .sub(Cmd::new("fourth").run(|a| captured_fourth = Some(a.to_vec())))
                .execute_with(args);

            (
                captured_first,
                captured_second,
                captured_third,
                captured_fourth,
            )
        };

        let args = vec![
            owned(&["second", "one"]),
            owned(&["second", "one", "two"]),
            owned(&["second", "one", "two", "three"]),
            owned(&["second", "one", "two", "three", "four"]),
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
                &a[1..],
                "arguments after the name of the subcommand must be passed to the subcommand"
            );
        }

        let args = vec![
            owned(&["third", "one"]),
            owned(&["third", "one", "two"]),
            owned(&["third", "one", "two", "three"]),
            owned(&["third", "one", "two", "three", "four"]),
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
                &a[1..],
                "arguments after the name of the subcommand must be passed to the subcommand"
            );
        }

        let args = vec![
            owned(&["fourth", "one"]),
            owned(&["fourth", "one", "two"]),
            owned(&["fourth", "one", "two", "three"]),
            owned(&["fourth", "one", "two", "three", "four"]),
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
            let (captured_first, captured_second, captured_third, captured_fourth) = run(&a);

            assert_eq!(
                captured_first,
                Some(a),
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

            App::new("first")
                .run(|a| captured_first = Some(a.to_vec()))
                .sub(
                    Cmd::new("second")
                        .run(|a| captured_second = Some(a.to_vec()))
                        .sub(Cmd::new("third").run(|a| captured_third = Some(a.to_vec())))
                        .sub(Cmd::new("fourth").run(|a| captured_fourth = Some(a.to_vec()))),
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
            owned(&["second", "third", "one"]),
            owned(&["second", "third", "one", "two"]),
            owned(&["second", "third", "one", "two", "three"]),
            owned(&["second", "third", "one", "two", "three", "four"]),
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
                &a[2..],
                "arguments after the name of the subcommand must be passed to the subcommand"
            );
        }

        let args = vec![
            owned(&["second", "fourth", "one"]),
            owned(&["second", "fourth", "one", "two"]),
            owned(&["second", "fourth", "one", "two", "three"]),
            owned(&["second", "fourth", "one", "two", "three", "four"]),
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
            assert_eq!(
                captured_first,
                Some(a),
                "all arguments should have been passed to first command"
            );
        }
    }
}
