use self::AstNode::*;
use pest::error::Error;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "lala.pest"]
pub struct LalaParser;


#[derive(PartialEq, Eq, Debug, Clone)]
pub enum MonadicVerb {
    Rank,
    Inverse,
    RREF,
    Transpose,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum DyadicVerb {
    Dot,
    Plus,
    Times,
}

#[derive(PartialEq, Debug, Clone)]
pub enum AstNode {
    Integer(i32),
    DoublePrecisionFloat(f64),
    MonadicOp {
        verb: MonadicVerb,
        expr: Box<AstNode>,
    },
    DyadicOp {
        verb: DyadicVerb,
        lhs: Box<AstNode>,
        rhs: Box<AstNode>,
    },
    Terms(Vec<AstNode>),
    Assignment {
        ident: String,
        expr: Box<AstNode>,
    },
    Ident(String),
    Matrix(Vec<Vec<AstNode>>),
}

fn build_ast_from_term(pair: pest::iterators::Pair<Rule>) -> AstNode {
    match pair.as_rule() {
        Rule::integer => {
            let istr = pair.as_str();
            let (sign, istr) = match &istr[..1] {
                "-" => (-1, &istr[1..]),
                _ => (1, &istr[..]),
            };
            let int: i32 = istr.parse().unwrap();
            AstNode::Integer(sign * int)
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
            AstNode::DoublePrecisionFloat(float)
        },
        Rule::expr => {
            build_ast_from_expr(pair).unwrap()
        },
        bad_term => panic!("Unexpected term: {:?}", bad_term),
    }
}

fn parse_monadic_verb(pair: pest::iterators::Pair<Rule>, expr: AstNode) -> Option<AstNode> {
    Some(AstNode::MonadicOp {
        verb: match pair.as_str() {
            "#" => MonadicVerb::Rank,
            "?" => MonadicVerb::Inverse,
            ">>" => MonadicVerb::RREF,
            ">+" => MonadicVerb::Transpose,
            _ => panic!("Monadic {} not supported (yet?)", pair.as_str()),
        },
        expr: Box::new(expr),
    })
}

fn parse_dyadic_verb(pair: pest::iterators::Pair<Rule>, lhs: AstNode, rhs: AstNode) -> Option<AstNode> {
    Some(AstNode::DyadicOp {
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        verb: match pair.as_str() {
            "@" => DyadicVerb::Dot,
            "++" => DyadicVerb::Plus,
            "**" => DyadicVerb::Times,
            _ => panic!("Dyadic {} not supported (yet?)", pair.as_str()),
        },
    })
}

fn build_ast_from_expr(pair: pest::iterators::Pair<Rule>) -> Option<AstNode> {
    match pair.as_rule() {
        Rule::expr => build_ast_from_expr(pair.into_inner().next()?),
        Rule::monadic => {
            let mut pair = pair.into_inner();
            let verb = pair.next()?;
            let op = build_ast_from_expr(pair.next()?)?;
            parse_monadic_verb(verb, op)
        },
        Rule::dyadic => {
            let mut pair = pair.into_inner();
            let lhs = build_ast_from_expr(pair.next()?)?;
            let op = pair.next()?;
            let rhs = build_ast_from_expr(pair.next()?)?;
            parse_dyadic_verb(op, lhs, rhs)
        },
        Rule::assn => {
            let mut pair = pair.into_inner();
            let ident = pair.next()?;
            let expr = build_ast_from_expr(pair.next()?)?;
            Some(AstNode::Assignment {
                ident: String::from(ident.as_str()),
                expr: Box::new(expr)
            })
        },
        Rule::ident => {
            let i = pair.as_str();
            Some(AstNode::Ident(i.to_string()))
        },
        Rule::terms => {
            let terms: Vec<AstNode> = pair.into_inner().map(build_ast_from_term).collect();
            Some(match terms.len() {
                1 => terms.get(0).unwrap().clone(),
                _ => Terms(terms),
            })
        },
        Rule::matrix => {
            let mut mat: Vec<Vec<AstNode>> = Vec::new();
            for row in pair.into_inner(){
                let terms: Vec<AstNode> = row.into_inner().map(build_ast_from_term).collect();
                mat.push(terms);
            }
            Some(Matrix(mat))
        }
        bad_expr => panic!("Unexpected expression: {:?}", bad_expr),
    }
}

pub fn parse(source: &str) -> Result<Vec<Box<AstNode>>, Error<Rule>> {
    let mut ast = vec![];

    let pairs = LalaParser::parse(Rule::program, source)?;
    for pair in pairs {
        match pair.as_rule() {
            Rule::expr => {
                ast.push(Box::new(build_ast_from_expr(pair).unwrap()));
            }
            _ => {}
        }
    }

    Ok(ast)
}