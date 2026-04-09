use std::fmt::Display;

trait StrExt {
    fn bound(&self) -> Bound;
    fn method(&self, arg: &'static str, aty: &'static str) -> Method;
}

impl StrExt for &'static str {
    fn bound(&self) -> Bound {
        Bound {
            most: self,
            last: self,
        }
    }

    fn method(&self, arg: &'static str, aty: &'static str) -> Method {
        Method {
            name: self,
            arg,
            aty,
        }
    }
}

struct Bound {
    most: &'static str,
    last: &'static str,
}

impl Bound {
    fn last(mut self, last: &'static str) -> Self {
        self.last = last;
        self
    }

    fn header(self, n: usize) -> Header {
        Header { bound: self, n }
    }
}

const LETTERS: &[char] = &['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L'];

struct Header {
    bound: Bound,
    n: usize,
}

impl Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "impl<")?;
        let last = LETTERS[self.n - 1];
        for c in LETTERS.iter().take(self.n - 1) {
            write!(f, "{c}: {}, ", self.bound.most)?;
        }
        write!(f, "{last}: {}", self.bound.last)?;
        write!(f, "> {} for (", self.bound.last)?;
        for c in LETTERS.iter().take(self.n - 1) {
            write!(f, "{c}, ")?;
        }
        write!(f, "{last}")?;
        write!(f, ")")?;
        Ok(())
    }
}

struct Method {
    name: &'static str,
    arg: &'static str,
    aty: &'static str,
}

impl Method {
    fn out(self, n: usize) -> OutMethod {
        OutMethod { method: self, n }
    }

    fn parse(self, n: usize, most: &'static str) -> ParseMethod {
        ParseMethod {
            method: self,
            n,
            most,
            last: most,
        }
    }
}

struct OutMethod {
    method: Method,
    n: usize,
}

impl Display for OutMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Method { name, arg, aty } = self.method;
        writeln!(f, "    fn {name}(&self, {arg}: {aty}) {{")?;
        for i in 0..self.n {
            writeln!(f, "        self.{i}.{name}({arg});")?;
        }
        writeln!(f, "    }}")?;
        Ok(())
    }
}

struct ParseMethod {
    method: Method,
    n: usize,
    most: &'static str,
    last: &'static str,
}

impl ParseMethod {
    fn last(mut self, last: &'static str) -> Self {
        self.last = last;
        self
    }
}

impl Display for ParseMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Method { name, arg, aty } = self.method;
        writeln!(
            f,
            "    fn {name}({}{arg}: {aty}) -> crate::Result<Self> {{",
            if self.most != self.last { "mut " } else { "" },
        )?;
        writeln!(f, "        Ok((")?;
        for _ in 0..self.n - 1 {
            writeln!(f, "            {arg}.{}()?,", self.most)?;
        }
        writeln!(f, "            {arg}.{}()?,", self.last)?;
        writeln!(f, "        ))")?;
        writeln!(f, "    }}")?;
        Ok(())
    }
}

struct Impl {
    header: Header,
    methods: Vec<Box<dyn Display>>,
}

fn join(s: impl IntoIterator<Item: Display>) -> String {
    s.into_iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

impl Display for Impl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {{", self.header)?;
        write!(f, "{}", join(&self.methods))?;
        writeln!(f, "}}")?;
        Ok(())
    }
}

fn per_n(n: usize) -> String {
    join([
        Impl {
            header: "ToOutput".bound().header(n),
            methods: vec![Box::new(
                "to_output".method("output", "&mut dyn Output").out(n),
            )],
        },
        Impl {
            header: "Inline".bound().last("Object").header(n),
            methods: vec![
                Box::new(
                    "accept_refs"
                        .method("visitor", "&mut impl RefVisitor")
                        .out(n),
                ),
                Box::new(
                    "parse"
                        .method("input", "Input")
                        .parse(n, "parse_inline")
                        .last("parse"),
                ),
            ],
        },
        Impl {
            header: "Inline".bound().header(n),
            methods: vec![Box::new(
                "parse_inline"
                    .method("input", "&mut Input")
                    .parse(n, "parse_inline"),
            )],
        },
        Impl {
            header: "ReflessInline".bound().last("ReflessObject").header(n),
            methods: vec![Box::new(
                "parse"
                    .method("input", "ReflessInput")
                    .parse(n, "parse_inline")
                    .last("parse"),
            )],
        },
        Impl {
            header: "ReflessInline".bound().header(n),
            methods: vec![Box::new(
                "parse_inline"
                    .method("input", "&mut ReflessInput")
                    .parse(n, "parse_inline"),
            )],
        },
    ])
}

fn main() {
    for i in 2..=12 {
        println!("{}", per_n(i));
    }
}
