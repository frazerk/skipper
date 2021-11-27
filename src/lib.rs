use std::env;

type Args = [String];

pub trait Execute {
    fn execute(self, args: &Args) -> bool;
}

pub trait Traverse {
    fn traverse(self, child: impl Execute);

    fn subcommand<F>(self, name: &str, run: F) -> SubTraverse<Self, F>
    where
        Self: Sized,
        F: FnOnce(&Args),
    {
        SubTraverse {
            name: name.into(),
            parent: self,
            run,
        }
    }
}

pub struct App<F>
where
    F: FnOnce(&Args),
{
    name: String,
    run: F,

    // TODO: Come up with better way of injecting this
    args: Vec<String>,
}

impl<F> Traverse for App<F>
where
    F: FnOnce(&Args),
{
    fn traverse(self, child: impl Execute) {
        match self.args.as_slice() {
            // If we don't have any arguments, execute the root regardless of the presence of
            // subcommands.
            a @ [] => {
                (self.run)(a);
            }
            // If we do have subcommands, only execute the root if none of the subcommands
            // matched.
            a @ [..] => {
                if !child.execute(a) {
                    (self.run)(a);
                }
            }
        }
    }
}

pub struct SubTraverse<P, F>
where
    P: Traverse,
    F: FnOnce(&Args),
{
    parent: P,
    run: F,
    name: String,
}

impl<P, F> Traverse for SubTraverse<P, F>
where
    P: Traverse,
    F: FnOnce(&Args),
{
    fn traverse(self, child: impl Execute) {
        self.parent.traverse(SubCommand {
            run: self.run,
            name: self.name,
            sub: child,
        });
    }
}

pub struct SubCommand<F, S> {
    run: F,
    name: String,
    sub: S,
}

impl<F, S> Execute for SubCommand<F, S>
where
    F: FnOnce(&Args),
    S: Execute,
{
    fn execute(self, args: &Args) -> bool {
        match args {
            // If there aren't any arguments, we don't match.
            [] => false,

            // TODO: clean this up, jesus christ
            [first, a @ ..] => {
                if first == &self.name {
                    if !self.sub.execute(a) {
                        (self.run)(a);
                    }

                    true
                } else {
                    false
                }
            }
        }
    }
}

pub struct SubCommandSet {}

pub struct Command<F>
where
    F: FnOnce(&Args),
{
    name: String,
    run: F,
}

impl<F> Execute for Command<F>
where
    F: FnOnce(&Args),
{
    fn execute(self, args: &Args) -> bool {
        match args {
            [] => false,
            [first, a @ ..] => {
                if first == &self.name {
                    (self.run)(a);
                    true
                } else {
                    false
                }
            }
        }
    }
}

pub struct NullCommand {}

impl Execute for NullCommand {
    fn execute(self, _: &Args) -> bool {
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
        let run = |args: Vec<String>| {
            let mut captured: Option<Vec<String>> = None;

            let cmd = App {
                name: "first".into(),
                run: |a| captured = Some(a.to_vec()),
                args,
            };

            cmd.traverse(NullCommand {});

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
            let captured = run(a.clone());

            // If we have a single command and no subcommands or flags, all arguments
            // must be passed to the top-level command.
            assert_eq!(captured, Some(a));
        }
    }

    #[test]
    fn one_level_subcommand_no_flags() {
        let run = |args: Vec<String>| {
            let mut captured_first: Option<Vec<String>> = None;
            let mut captured_second: Option<Vec<String>> = None;

            let cmd = App {
                name: "first".into(),
                run: |a| captured_first = Some(a.to_vec()),
                args,
            }
            .subcommand("second", |a| captured_second = Some(a.to_vec()));

            cmd.traverse(NullCommand {});

            (captured_first, captured_second)
        };

        let args = vec![
            owned(&["second", "one"]),
            owned(&["second", "one", "two"]),
            owned(&["second", "one", "two", "three"]),
            owned(&["second", "one", "two", "three", "four"]),
        ];

        for a in args {
            let (captured_first, captured_second) = run(a.clone());

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
            let (captured_first, captured_second) = run(a.clone());

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

            let cmd = App {
                name: "first".into(),
                run: |a| captured_first = Some(a.to_vec()),
                args: args.clone(),
            }
            .subcommand("second", |a| captured_second = Some(a.to_vec()))
            .subcommand("third", |a| captured_third = Some(a.to_vec()));

            cmd.traverse(NullCommand {});

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
