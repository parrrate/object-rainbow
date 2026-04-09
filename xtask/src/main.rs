use std::fmt::Display;

trait StrExt {
    fn bound(&self) -> Bound;
    fn method(&self, arg: &'static str, aty: &'static str) -> Method;
    fn co(&self) -> Const;
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

    fn co(&self) -> Const {
        Const { name: self }
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
        Header {
            bound: self,
            n,
            prefix: "",
            suffix: "".into(),
        }
    }
}

const LETTERS: &[char] = &['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L'];

struct Header {
    bound: Bound,
    n: usize,
    prefix: &'static str,
    suffix: String,
}

impl Header {
    fn prefix(mut self, prefix: &'static str) -> Self {
        self.prefix = prefix;
        self
    }

    fn suffix(mut self, suffix: impl Display) -> Self {
        self.suffix = suffix.to_string();
        self
    }
}

impl Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "impl<{}", self.prefix)?;
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
        write!(f, ") {}", self.suffix)?;
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

struct WhereFoldAdd {
    n: usize,
}

impl Display for WhereFoldAdd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "where tarr![")?;
        for c in LETTERS.iter().take(self.n) {
            write!(f, "{c},")?;
        }
        write!(f, "]: typenum::FoldAdd<Output: Unsigned>")?;
        Ok(())
    }
}

struct SizeFoldAdd {
    n: usize,
}

impl Display for SizeFoldAdd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type Size = <tarr![")?;
        for c in LETTERS.iter().take(self.n) {
            write!(f, "{c},")?;
        }
        writeln!(f, "] as typenum::FoldAdd>::Output;")?;
        Ok(())
    }
}

struct Const {
    name: &'static str,
}

impl Const {
    fn add(self, n: usize, ty: &'static str) -> AddConst {
        AddConst { co: self, n, ty }
    }

    fn tags(self, n: usize) -> TagsConst {
        TagsConst { co: self, n }
    }
}

struct AddConst {
    co: Const,
    n: usize,
    ty: &'static str,
}

impl Display for AddConst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "    const {}: {} = {};",
            self.co.name,
            self.ty,
            LETTERS
                .iter()
                .take(self.n)
                .map(|c| format!("{c}::{}", self.co.name))
                .collect::<Vec<_>>()
                .join(" + "),
        )
    }
}

struct TagsConst {
    co: Const,
    n: usize,
}

impl Display for TagsConst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "    const {}: Tags = Tags(&[], &[{}]);",
            self.co.name,
            LETTERS
                .iter()
                .take(self.n)
                .map(|c| format!("&{c}::{}", self.co.name))
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}

struct Impl {
    header: Header,
    members: Vec<Box<dyn Display>>,
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
        write!(f, "{}", join(&self.members))?;
        writeln!(f, "}}")?;
        Ok(())
    }
}

fn per_n(n: usize) -> String {
    join([
        Impl {
            header: "ToOutput".bound().header(n),
            members: vec![Box::new(
                "to_output".method("output", "&mut dyn Output").out(n),
            )],
        },
        Impl {
            header: "Topological".bound().last("Topological").header(n),
            members: vec![Box::new(
                "accept_points"
                    .method("visitor", "&mut impl PointVisitor")
                    .out(n),
            )],
        },
        Impl {
            header: "Tagged".bound().header(n),
            members: vec![Box::new("TAGS".co().tags(n))],
        },
        Impl {
            header: "Inline".bound().last("Object").header(n),
            members: vec![],
        },
        Impl {
            header: "Inline".bound().header(n),
            members: vec![],
        },
        Impl {
            header: "ReflessInline".bound().last("ReflessObject").header(n),
            members: vec![],
        },
        Impl {
            header: "ReflessInline".bound().header(n),
            members: vec![],
        },
        Impl {
            header: "Size".bound().header(n).suffix(WhereFoldAdd { n }),
            members: vec![
                Box::new("SIZE".co().add(n, "usize")),
                Box::new(SizeFoldAdd { n }),
            ],
        },
        Impl {
            header: "ParseInline<II>"
                .bound()
                .last("Parse<II>")
                .header(n)
                .prefix("II: ParseInput,"),
            members: vec![Box::new(
                "parse"
                    .method("input", "II")
                    .parse(n, "parse_inline")
                    .last("parse"),
            )],
        },
        Impl {
            header: "ParseInline<II>"
                .bound()
                .header(n)
                .prefix("II: ParseInput,"),
            members: vec![Box::new(
                "parse_inline"
                    .method("input", "&mut II")
                    .parse(n, "parse_inline"),
            )],
        },
    ])
}

fn main() {
    println!("use typenum::tarr;");
    println!();
    println!("use crate::*;");
    println!();

    for i in 2..=12 {
        if i > 2 {
            println!();
        }
        print!("{}", per_n(i));
    }
}
