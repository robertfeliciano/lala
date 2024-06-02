use self::AstNode::*;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use anyhow::anyhow;

#[derive(Parser)]
#[grammar = "lala.pest"]
pub struct LalaParser;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum MonadicVerb {
    Rank,
    Inverse,
    RREF,
    Transpose,
    Determinant,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum DyadicVerb {
    Dot,
    Plus,
    Times,
}

#[derive(PartialEq, Debug, Clone)]
pub enum AstNode<'a> {
    Integer(i32),
    DoublePrecisionFloat(f64),
    MonadicOp {
        verb: MonadicVerb,
        expr: Box<AstNode<'a>>,
    },
    DyadicOp {
        verb: DyadicVerb,
        lhs: Box<AstNode<'a>>,
        rhs: Box<AstNode<'a>>,
    },
    Terms(Vec<AstNode<'a>>),
    Assignment {
        ident: String,
        expr: Box<AstNode<'a>>,
    },
    Ident(String),
    Matrix(Vec<Vec<AstNode<'a>>>),
    Command((&'a str, Vec<&'a str>)),
    Fun((String, Vec<AstNode<'a>>, Vec<AstNode<'a>>)),
    App((String, Vec<AstNode<'a>>)),
}

fn build_ast_from_term(pair: Pair<Rule>) -> Option<AstNode> {
    match pair.as_rule() {
        Rule::integer => {
            let istr = pair.as_str();
            let (sign, istr) = match &istr[..1] {
                "-" => (-1, &istr[1..]),
                _ => (1, &istr[..]),
            };
            let int: i32 = istr.parse().unwrap();
            Some(AstNode::Integer(sign * int))
        },
        Rule::decimal => {
            let dstr = pair.as_str();
            let (sign, dstr) = match &dstr[..1] {
                "-" => (-1.0, &dstr[1..]),
                _ => (1.0, &dstr[..]),
            };
            let mut float: f64 = dstr.parse().unwrap();
            if float != 0.0 {
                float *= sign;
            }
            Some(AstNode::DoublePrecisionFloat(float))
        },
        Rule::expr => build_ast_from_expr(pair),
        _bad_term => None,
    }
}

fn parse_monadic_verb<'a>(pair: Pair<Rule>, expr: AstNode<'a>) -> Option<AstNode<'a>> {
    let verb = match pair.as_str() {
        "#" => MonadicVerb::Rank,
        "?" => MonadicVerb::Inverse,
        "rref" => MonadicVerb::RREF,
        "%" => MonadicVerb::Transpose,
        "det" => MonadicVerb::Determinant,
        _ => return None,
    };

    Some(AstNode::MonadicOp {
        verb,
        expr: Box::new(expr),
    })
}

fn parse_dyadic_verb<'a>(
    pair: Pair<Rule>,
    lhs: AstNode<'a>,
    rhs: AstNode<'a>,
) -> Option<AstNode<'a>> {
    let verb = match pair.as_str() {
        "@" => DyadicVerb::Dot,
        "++" => DyadicVerb::Plus,
        "**" => DyadicVerb::Times,
        _ => return None
    };
    
    Some(AstNode::DyadicOp {
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        verb
    })
}

fn parse_cmd<'a>(cmd: Pair<'a, Rule>, cmd_params: Option<Pairs<'a, Rule>>) -> Option<AstNode<'a>> {
    let params: Vec<&str> = match cmd_params {
        Some(p) => p.into_iter().map(|s| s.as_str()).collect(),
        None => vec![],
    };

    Some(AstNode::Command((cmd.as_str(), params)))
}

fn build_ast_from_expr(pair: Pair<Rule>) -> Option<AstNode> {
    match pair.as_rule() {
        Rule::expr => build_ast_from_expr(pair.into_inner().next()?),
        Rule::command => {
            let mut pair = pair.into_inner();
            let cmd = pair.next()?;
            if cmd.as_str() == "dbg" {
                return parse_cmd(cmd, None);
            }
            let cmd_params = pair.next()?.into_inner();
            parse_cmd(cmd, Some(cmd_params))
        }
        Rule::monadic => {
            let mut pair = pair.into_inner();
            let verb = pair.next()?;
            let op = build_ast_from_expr(pair.next()?)?;
            parse_monadic_verb(verb, op)
        }
        Rule::dyadic => {
            let mut pair = pair.into_inner();
            let lhs = build_ast_from_expr(pair.next()?)?;
            let op = pair.next()?;
            let rhs = build_ast_from_expr(pair.next()?)?;
            parse_dyadic_verb(op, lhs, rhs)
        }
        Rule::assn => {
            let mut pair = pair.into_inner();
            let ident = pair.next()?;
            let expr = build_ast_from_expr(pair.next()?)?;
            Some(AstNode::Assignment {
                ident: String::from(ident.as_str()),
                expr: Box::new(expr),
            })
        }
        Rule::ident => {
            let i = pair.as_str();
            Some(AstNode::Ident(i.to_string()))
        }
        Rule::terms => {
            let unparsed_terms = pair.into_inner();
            let mut terms: Vec<AstNode> = Vec::new();
            for ut in unparsed_terms {
                let term = match build_ast_from_term(ut) {
                    Some(t) => t,
                    None => return None
                };
                terms.push(term);
            }
            Some(match terms.len() {
                1 => terms.get(0).unwrap().clone(),
                _ => Terms(terms),
            })
        }
        Rule::matrix => {
            let mut mat: Vec<Vec<AstNode>> = Vec::new();
            for row in pair.into_inner() {
                let mut terms: Vec<AstNode> = Vec::new();
                for ut in row.into_inner() {
                    let term = match build_ast_from_term(ut) {
                        Some(t) => t,
                        None => return None
                    };
                    terms.push(term);
                }
                mat.push(terms);
            }
            Some(Matrix(mat))
        }
        Rule::fun_decl => {
            let mut pair = pair.into_inner();
            let ident = pair.next()?;
            let mut params: Vec<AstNode> = Vec::new();
            let unparsed = pair.next()?.into_inner();
            for param in unparsed {
                if let Some(AstNode::Ident(id)) = build_ast_from_expr(param) {
                    params.push(AstNode::Ident(id));
                } else {
                    continue;
                }
            }
            let body = pair.next()?;
            let parsed_body: Vec<AstNode> = body
                .into_inner()
                .map(build_ast_from_expr)
                .collect::<Option<Vec<_>>>()?;

            Some(Fun((
                ident.as_span().as_str().to_string(),
                params,
                parsed_body,
            )))
        }
        Rule::app => {
            let mut pair = pair.into_inner();
            let ident = pair.next()?;
            let mut parsed_params = Vec::new();
            while let Some(param) = pair.next() {
                parsed_params.push(build_ast_from_expr(param)?);
            }
            Some(App((ident.as_span().as_str().to_string(), parsed_params)))
        }
        _bad_expr => None
    }
}

pub fn parse(source: &str) -> Result<Vec<Box<AstNode>>, anyhow::Error> {
    let mut ast = vec![];

    let pairs = LalaParser::parse(Rule::program, source)?;
    for pair in pairs {
        match pair.as_rule() {
            Rule::fun_decl | Rule::expr => {
                let node = match build_ast_from_expr(pair) {
                    Some(n) => n,
                    None => return Err(anyhow!("Parse error! Please consult the guide :)"))
                };
                ast.push(Box::new(node));
            }
            _ => {}
        }
    }

    Ok(ast)
}