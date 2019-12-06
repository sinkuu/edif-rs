use crate::atom::Atom;
use combine::stream::state::SourcePosition;
use combine::*;
use std::fmt;

#[derive(Debug)]
pub enum ExprKind {
    List(Vec<Expr>),
    Symbol(Atom),
    Str(String),
    Num(i32),
}

#[derive(Clone, Copy)]
pub struct Pos(pub i32, pub i32);

impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub pos: Pos,
}

impl Expr {
    pub fn list(&self) -> Option<&[Expr]> {
        match &self.kind {
            ExprKind::List(es) => Some(&es),
            _ => None,
        }
    }

    pub fn symbol(&self) -> Option<Atom> {
        match &self.kind {
            ExprKind::Symbol(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn num(&self) -> Option<i32> {
        match self.kind {
            ExprKind::Num(n) => Some(n),
            _ => None,
        }
    }

    pub fn str(&self) -> Option<&str> {
        match &self.kind {
            ExprKind::Str(n) => Some(&*n),
            _ => None,
        }
    }
}

parser! {
    pub fn sexpr_parser['a, I]()(I) -> Expr
    where [I: combine::Stream<Item=char> +
        combine::RangeStream +
        combine::StreamOnce<Range = &'a str, Position = SourcePosition>]
    {
        use combine::parser::char::{char as cmb_char, letter, spaces};
        use combine::parser::range;
        use combine::{between, many, position};

        let num = range::take_while1(|c: char| c.is_ascii_digit())
            .map(|ds: &str| ExprKind::Num(ds.parse::<i32>().unwrap()));
        let string = cmb_char('"').with(range::take_while(|c: char| c != '"')).skip(cmb_char('"'))
            .map(|s: &str| ExprKind::Str(s.to_string()));
        let list = between(
            cmb_char('('),
            cmb_char(')'),
            many(sexpr_parser()),
        )
        .map(|es| ExprKind::List(es));
        let symbol = letter()
            .or(cmb_char('|'))
            .or(cmb_char('&'))
            .and(range::take_while(|c: char| {
                c.is_ascii_alphanumeric() || c == '_' || c == '&'
            }))
            .map(|(c, cs): (char, &str)| {
                let mut s = String::with_capacity(cs.len() + 1);
                s.push(c);
                s.push_str(cs);
                ExprKind::Symbol(Atom::from(s))
            });

        spaces()
            .with(position())
            .and(choice!(num, string, symbol, list))
            .map(|(pos, kind): (SourcePosition, ExprKind)| {
                Expr {
                    kind,
                    pos: Pos(pos.line, pos.column),
                }
            })
            .skip(spaces())
    }
}
