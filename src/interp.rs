use super::parser::*;
use super::types::*;
use anyhow::{anyhow, Error};
use std::{collections::HashMap, ops::Deref};

#[inline]
fn get_value<'a>(
    map: &mut HashMap<String, LalaType<'a>>,
    key: &'a String,
) -> Result<LalaType<'a>, Error> {
    match map.get(key) {
        Some(val) => Ok(val.clone()),
        None => Err(anyhow!("Key not found in the hashmap: {}", key)),
    }
}

#[inline]
fn eval_expr<'a, 'b>(
    env: &mut HashMap<String, LalaType<'a>>,
    expr: &'b AstNode<'b>,
    func: &str,
) -> Result<LalaType<'a>, Error>
where
    'b: 'a,
{
    match expr {
        AstNode::Ident(id) => get_value(env, id),
        AstNode::MonadicOp { verb, expr } => eval_monadic_op(expr, env, verb),
        AstNode::DyadicOp { verb, lhs, rhs } => eval_dyadic_op(lhs, rhs, env, verb),
        AstNode::Matrix(m) => Ok(LalaType::Matrix(construct_matrix(m)?)),
        _ => Err(anyhow!("error processing {func} consult the docs.")),
    }
}

fn eval_monadic_op<'a, 'expr>(
    expr: &'expr AstNode<'expr>,
    env: &mut HashMap<String, LalaType<'a>>,
    verb: &'a MonadicVerb,
) -> Result<LalaType<'a>, Error>
where
    'expr: 'a,
{
    let func = verb.to_string();
    let matrix = match eval_expr(env, expr, &func)? {
        LalaType::Matrix(mat) => mat,
        _ => {
            return Err(anyhow!(
                "monadic op {} cna only be used on a matrix",
                verb.to_string()
            ))
        }
    };
    Ok(match verb {
        MonadicVerb::Inverse => {
            match matrix.inverse() {
                Ok(result) => LalaType::Matrix(result),
                Err(e) => return Err(e),
            }
        },
        MonadicVerb::Rank => LalaType::Integer(matrix.rank()),
        MonadicVerb::RREF => LalaType::Matrix(matrix.rref()),
        MonadicVerb::Transpose => LalaType::Matrix(matrix.transpose()),
        MonadicVerb::Determinant => {
            match matrix.det() {
                Ok(result) => LalaType::Double(result),
                Err(e) => return Err(e),
            }
        },
    })
}

fn eval_dyadic_op<'a, 'lhs, 'rhs>(
    lhs: &'lhs AstNode<'lhs>,
    rhs: &'rhs AstNode<'rhs>,
    env: &mut HashMap<String, LalaType<'a>>,
    verb: &'a DyadicVerb,
) -> Result<LalaType<'a>, Error>
where
    'lhs: 'a,
    'rhs: 'a,
{
    let func = verb.to_string();
    let leftside = if let LalaType::Matrix(left) = eval_expr(env, lhs, &func)? {
        left
    } else {
        return Err(anyhow!("can only call {func} on a matrix"));
    };
    let rightside = if let LalaType::Matrix(right) = eval_expr(env, rhs, &func)? {
        right
    } else {
        return Err(anyhow!("can only call {func} on a matrix"));
    };
    Ok(match verb {
        DyadicVerb::Dot => {
            match leftside.dot(rightside.clone()) {
                Ok(m) => LalaType::Matrix(m),
                Err(e) => return Err(e),
            }
        },
        DyadicVerb::Plus => {
            match leftside.combine(rightside, |a, b| a + b) {
                Ok(result) => LalaType::Matrix(result),
                Err(e) => return Err(e),
            }
        },
        DyadicVerb::Times => {
            match leftside.combine(rightside, |a, b| a * b) {
                Ok(result) => LalaType::Matrix(result),
                Err(e) => return Err(e),
            }
        },
    })
}

fn eval_assignment<'a, 'b>(
    ident: &'a String,
    expr: &'b Box<AstNode<'b>>,
    env: &mut HashMap<String, LalaType<'a>>,
) -> Result<(), Error>
where
    'b: 'a,
{
    match expr.deref() {
        AstNode::Integer(scalar) => match env.insert(ident.to_string(), LalaType::Integer(*scalar))
        {
            _ => Ok(()),
        },
        AstNode::DoublePrecisionFloat(scalar) => {
            match env.insert(ident.to_string(), LalaType::Double(*scalar)) {
                _ => Ok(()),
            }
        }
        AstNode::Ident(rhs_ident) => {
            let val = match env.get(rhs_ident) {
                Some(v) => v,
                None => return Err(anyhow!("{rhs_ident} referenced before definition."))
            };
            match env.insert(ident.to_string(), val.clone()) {
                _ => Ok(()),
            }
        }
        AstNode::Matrix(v) => {
            let mat = construct_matrix(v)?;
            match env.insert(ident.to_string(), LalaType::Matrix(mat)) {
                _ => Ok(()),
            }
        }
        AstNode::MonadicOp { verb, expr } => {
            let result = eval_monadic_op(expr, env, verb)?;
            match env.insert(ident.to_string(), result) {
                _ => Ok(()),
            }
        }
        AstNode::DyadicOp { verb, lhs, rhs } => {
            let result = eval_dyadic_op(lhs, rhs, env, verb)?;
            match env.insert(ident.to_string(), result) {
                _ => Ok(()),
            }
        }
        AstNode::App((name, params)) => {
            let result = match interp_app(name, params, env) {
                Ok(val) => val,
                Err(e) => return Err(e),
            };
            match env.insert(ident.to_string(), result) {
                _ => Ok(()),
            }
        }
        _ => Err(anyhow!("interpreter error!")),
    }
}

fn interp_fun<'a>(
    name: &String,
    params: &Vec<AstNode<'a>>,
    body: &Vec<AstNode<'a>>,
    env: &mut HashMap<String, LalaType<'a>>,
) -> () {
    match env.insert(
        name.to_string(),
        LalaType::Fun((name.to_string(), params.to_vec(), body.to_vec())),
    ) {
        _ => (),
    }
}

fn interp_app<'a, 'b>(
    name: &'a String,
    params: &'a Vec<AstNode<'a>>,
    env: &HashMap<String, LalaType<'b>>,
) -> Result<LalaType<'b>, Error>
where
    'a: 'b,
{
    let (_, aliases, body_o) = match env.get(name) {
        Some(LalaType::Fun((n, a, b))) => (n, a, b),
        _ => {
            return Err(anyhow!("Function {name} referenced before definition"));
        }
    };
    if params.len() != aliases.len() {
        return Err(anyhow!(
            "{name} supplied incorrect number of arguments. Expected {}, found {}",
            aliases.len(),
            params.len()
        ));
    }
    let mut function_scope = env.clone();
    for (provided_node, alias_node) in params.iter().zip(aliases.iter()) {
        // aliases are the parameter names in the function signature
        // we need to bind the value of the provided params in the function call to these aliases
        // for the scope of the function

        // get the identifier alias for the current parameter
        let alias = match alias_node {
            AstNode::Ident(i) => i.to_owned(),
            _ => {
                return Err(anyhow!("how the hell"));
            }
        };

        // evaluate the provided parameter
        let provided = match provided_node {
            AstNode::Integer(int) => LalaType::Integer(*int),
            AstNode::DoublePrecisionFloat(d) => LalaType::Double(*d),
            AstNode::MonadicOp { verb, expr } => {
                eval_monadic_op(&expr, &mut function_scope, &verb)?
            }
            AstNode::DyadicOp { verb, lhs, rhs } => {
                eval_dyadic_op(&lhs, &rhs, &mut function_scope, &verb)?
            }
            AstNode::Ident(i) => match function_scope.get(i) {
                Some(val) => val.clone(),
                None => {
                    return Err(anyhow!("identifer {i} referenced before definition"));
                }
            },
            AstNode::Matrix(m) => {
                if let Ok(mat) = construct_matrix(&m) {
                    LalaType::Matrix(mat)
                } else {
                    return Err(anyhow!("problem passing matrix to function..."));
                }
            }
            AstNode::App((func_name, func_params)) => {
                let temp =
                    if let Ok(intermediate) = interp_app(func_name, func_params, &function_scope) {
                        intermediate
                    } else {
                        return Err(anyhow!(
                            "issue applying function {func_name} as a parameter"
                        ));
                    };
                temp
            }
            _ => {
                return Err(anyhow!("interpreter error..."));
            }
        };

        // add the value of the parameter under the alias for the function's scope
        function_scope.insert(alias, provided);
    }

    // now that the parameter values have been assigned, we just need to interpret the
    // body of the function and return the result of the last expression

    let body = body_o.to_owned().leak();
    // function body can only contain assignment of variables and functions
    for expr in body[0..body.len() - 1].iter() {
        match expr {
            AstNode::Assignment { ident, expr } => {
                match eval_assignment(ident, expr, &mut function_scope) {
                    Ok(_) => (),
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            AstNode::Fun((name, params, body)) => {
                interp_fun(name, params, body, &mut function_scope)
            }
            _ => {
                return Err(anyhow!(
                    "function body only allows assigment and function declarations"
                ));
            }
        }
    }

    let another_body = body_o.to_owned();
    let last_expr = match another_body.last() {
        Some(res) => res.to_owned(),
        None => {
            return Err(anyhow!("empty function body somehow!"));
        }
    };

    let final_result = match last_expr {
        // FUNCTIONS MUST END WITH IDENTIFIERS AS THE RETURN VALUE
        AstNode::Ident(id) => match function_scope.get(&id) {
            Some(val) => val.to_owned(),
            None => todo!(),
        },
        _ => {
            return Err(anyhow!("return statement must only be an identifier"));
        }
    };

    Ok(final_result)
}

pub fn interp<'a>(
    ast: &'a [Box<AstNode<'_>>],
    map: Option<&mut HashMap<String, LalaType<'a>>>,
    tcp: bool,
) -> Result<String, Error> {
    let mut binding = HashMap::new();
    #[allow(unused_mut)]
    let mut env: &mut HashMap<String, LalaType> = match map {
        Some(m) => m,
        None => &mut binding,
    };

    let mut result = String::new();

    for node in ast {
        let _ = match node.deref() {
            AstNode::Assignment { ident, expr } => {
                let _ = eval_assignment(ident, expr, env)?;
                result = if tcp {
                    format!("{}", env.get(ident).unwrap())
                } else {
                    result
                };
            }
            AstNode::MonadicOp { verb, expr } => {
                let res = eval_monadic_op(expr, env, verb)?;
                result = if tcp { format!("{}", res) } else { result };
            }
            AstNode::DyadicOp { verb, lhs, rhs } => {
                let res = eval_dyadic_op(lhs, rhs, env, verb)?;
                result = if tcp { format!("{}", res) } else { result };
            }
            AstNode::Ident(var) => {
                let value = match env.get(var) {
                    Some(v) => v,
                    None => return Err(anyhow!("{var} referenced before definition."))
                };
                result = format!("{}", value);
            }
            AstNode::Fun((name, params, body)) => {
                interp_fun(name, params, body, env);
                result = format!("fun {name} added to env");
            }
            AstNode::App((name, params)) => {
                result = match interp_app(name, params, &env.clone()) {
                    Ok(evaluated) => evaluated.to_string(),
                    Err(e) => return Err(e),
                }
            }
            bad_line => return Ok(format!("Invalid line: {:?}", bad_line)),
        };
    }

    Ok(result)
}
