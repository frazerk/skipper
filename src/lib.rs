use once_cell::sync::Lazy;
use std::env;

static ARGS: Lazy<Vec<String>> = Lazy::new(|| std::env::args().collect());

type Args = [String];

pub trait Visitor {
    // TODO: rename
    fn exec(self, args: &Args) -> bool;
}

// TODO: figure out how to remove duplication or simplify forwarding to Cmd
pub struct App<'a, F, N> {
    cmd: Cmd<'a, F, N>,
}

impl<'a> App<'a, (), Leaf> {
    pub fn new(name: &'a str) -> Self {
        Self {
            cmd: Cmd::root2(name, ARGS.as_slice()),
        }
    }
}

impl<'a, N> App<'a, (), N> {
    pub fn run<F>(self, run: F) -> App<'a, F, N>
    where
        F: FnOnce(&Args),
    {
        App {
            cmd: self.cmd.run(run),
        }
    }
}

impl<'a, F, N> App<'a, F, N> {
    pub fn sub2<T, G, V>(self, name: &'a str, sub: T) -> App<'a, F, Chain<CmdTree<'a, G, V>, N>>
    where
        T: FnOnce(Cmd<'a, (), Leaf>) -> Cmd<'a, G, V>,
    {
        App {
            cmd: self.cmd.sub2(name, sub),
        }
    }
}

impl<'a, F, N> App<'a, F, N>
where
    CmdTree<'a, F, N>: Visitor,
    F: FnOnce(&Args), // TODO: this constraint will cause problems
    N: Visitor,
{
    pub fn execute(self) {
        self.execute_with(&ARGS)
    }

    pub fn execute_with(self, args: &Args) {
        let mut run_help = false;
        let ran;

        let data = {
            let c = self.cmd.subcmd(Cmd::new("help").run(|_| run_help = true));
            ran = c.tree.exec(args);
            c.data
        };

        if !ran || run_help {
            println!("{:#?}", data);
        }
    }
}

// TODO: more specific name
#[derive(Default, Debug)]
struct Data<'a> {
    // TODO: should this be in Data or Cmd?
    args: &'a Args,

    name: &'a str,
    description: Option<&'a str>,

    sub: Vec<Data<'a>>,
}

pub struct Leaf;

impl Visitor for Leaf {
    fn exec(self, _: &Args) -> bool {
        false
    }
}

pub struct Cmd<'a, F, N> {
    data: Data<'a>,
    tree: CmdTree<'a, F, N>,
}

// TODO: delete this impl block, execute is on App now
impl<'a, F, N> Cmd<'a, F, N>
where
    CmdTree<'a, F, N>: Visitor,
    F: FnOnce(&Args), // TODO: this constraint will cause problems
    N: Visitor,
{
    pub fn execute(self) {
        let args: Vec<String> = env::args().collect();

        self.execute_with(&args)
    }

    pub fn execute_with(self, args: &Args) {
        let mut run_help = false;
        let ran;

        let data = {
            let c = self.subcmd(Cmd::new("help").run(|_| run_help = true));
            ran = c.tree.exec(args);
            c.data
        };

        if !ran || run_help {
            println!("{:#?}", data);
        }
    }
}

impl<'a, N> Cmd<'a, (), N> {
    pub fn run<F>(self, run: F) -> Cmd<'a, F, N>
    where
        F: FnOnce(&Args),
    {
        Cmd {
            tree: self.tree.run(run),
            data: self.data,
        }
    }
}

impl<'a, F, N> Cmd<'a, F, N> {
    // TODO: find shorter name (sub conflicts with common trait)
    pub fn subcmd<G, V>(mut self, sub: Cmd<'a, G, V>) -> Cmd<'a, F, Chain<CmdTree<G, V>, N>> {
        self.data.sub.push(sub.data);

        Cmd {
            tree: self.tree.chain(sub.tree),
            data: self.data,
        }
    }

    pub fn flag(mut self, name: &str, value: &mut u32) -> Self {
        match self.data.args {
            [first, second, a @ ..] => {
                if first == name {
                    match second.parse::<u32>() {
                        Ok(i) => {
                            self.data.args = a;
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

    pub fn sub2<T, G, V>(mut self, name: &'a str, sub: T) -> Cmd<'a, F, Chain<CmdTree<'a, G, V>, N>>
    where
        T: FnOnce(Cmd<'a, (), Leaf>) -> Cmd<'a, G, V>,
    {
        let cmd = match self.data.args {
            [] => Cmd::new2(name, &[]),
            [_, a @ ..] => Cmd::new2(name, a),
        };

        let sub = (sub)(cmd);

        self.data.sub.push(sub.data);

        Cmd {
            tree: self.tree.chain(sub.tree),
            data: self.data,
        }
    }

    pub fn description(mut self, description: &'a str) -> Self {
        self.data.description = Some(description);
        self
    }
}

impl<'a> Cmd<'a, (), Leaf> {
    fn build(name: &'a str, root: bool, args: &'a Args) -> Self {
        Self {
            tree: CmdTree {
                name,
                root,
                run: (),
                next: Leaf,
            },
            data: Data {
                name,
                args,
                ..Data::default()
            },
        }
    }

    pub fn new2(name: &'a str, args: &'a Args) -> Self {
        Self::build(name, false, args)
    }

    pub fn new(name: &'a str) -> Self {
        Self::build(name, false, &[])
    }
    pub fn root2(name: &'a str, args: &'a Args) -> Self {
        Self::build(name, true, args)
    }

    pub fn root(name: &'a str) -> Self {
        Self::build(name, true, &[])
    }
}

pub struct Chain<C, N> {
    cur: C,
    next: N,
}

impl<C, N> Visitor for Chain<C, N>
where
    C: Visitor,
    N: Visitor,
{
    fn exec(self, args: &Args) -> bool {
        if self.cur.exec(args) {
            true
        } else {
            self.next.exec(args)
        }
    }
}

pub struct CmdTree<'a, F, N> {
    name: &'a str,
    run: F,
    next: N,

    // TODO: get rid of this
    root: bool,
}

impl<'a, F, N> CmdTree<'a, F, N> {
    pub fn run<G>(self, run: G) -> CmdTree<'a, G, N>
    where
        G: FnOnce(&Args),
    {
        CmdTree {
            run,
            root: self.root,
            name: self.name,
            next: self.next,
        }
    }

    pub fn chain<G, V>(self, sub: CmdTree<'a, G, V>) -> CmdTree<'a, F, Chain<CmdTree<G, V>, N>> {
        CmdTree {
            name: self.name,
            run: self.run,
            root: self.root,
            next: Chain {
                cur: sub,
                next: self.next,
            },
        }
    }

    fn check_run<'b>(&self, args: &'b Args) -> Option<&'b Args> {
        match args {
            [] => None,
            [first, a @ ..] => {
                if first == self.name || self.root {
                    Some(a)
                } else {
                    None
                }
            }
        }
    }
}

impl<'a, F, N> Visitor for CmdTree<'a, F, N>
where
    F: FnOnce(&Args), // TODO: can we get this to be FnOnce?
    N: Visitor,
{
    fn exec(self, args: &Args) -> bool {
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

impl<'a, N> Visitor for CmdTree<'a, (), N>
where
    N: Visitor,
{
    fn exec(self, args: &Args) -> bool {
        match self.check_run(args) {
            Some(args) => {
                self.next.exec(args);
                true
            }
            None => false,
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
