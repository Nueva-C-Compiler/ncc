use crate::syntax::declaration::Declaration;
use crate::syntax::expression::Expression;
use crate::syntax::statement::{Statement, StatementContents};
use crate::syntax::Node;
use std::collections::HashMap;
use std::rc::Rc;

pub fn resolve_identifiers(program: Vec<Rc<Declaration>>) {
    let mut globals = HashMap::new();
    for declaration in program {
        resolve_declaration_identifiers(&declaration, &mut globals);
    }
}

fn resolve_block_identifiers(
    block: &Vec<Node>,
    working_identifiers: &HashMap<u64, Rc<Declaration>>,
) {
    let mut locals = working_identifiers.clone();
    for node in block {
        match node {
            Node::Declaration(declaration) => {
                resolve_declaration_identifiers(&declaration, &mut locals)
            }
            Node::Statement(statement) => resolve_statement_identifiers(&statement, &locals),
        }
    }
}

fn resolve_declaration_identifiers(
    declaration: &Rc<Declaration>,
    working_identifiers: &mut HashMap<u64, Rc<Declaration>>,
) {
    let mut locals = working_identifiers.clone();
    match &**declaration {
        Declaration::Variable(var_decl) => {
            locals.insert(var_decl.identifier.hash, declaration.clone());
            if let Some(expression_refcell) = &var_decl.initializer {
                let mut expression = expression_refcell.borrow_mut();
                resolve_expression_identifiers(&mut *expression, &locals);
            }
        }
        Declaration::Function(fun_decl) => {
            locals.insert(fun_decl.identifier.hash, declaration.clone());
            if let Some(body) = &fun_decl.body {
                resolve_block_identifiers(body, &locals);
            }
        }
    }
}

fn resolve_statement_identifiers(
    statement: &Statement,
    working_identifiers: &HashMap<u64, Rc<Declaration>>,
) {
    let mut statement_contents = statement.statement_contents.borrow_mut();
    match &mut *statement_contents {
        StatementContents::Compound(substatements) => {
            resolve_block_identifiers(&substatements, &working_identifiers)
        }
        StatementContents::Expression(ref mut expression) => {
            resolve_expression_identifiers(expression, &working_identifiers)
        }
        StatementContents::IfStatement(ref mut if_statement) => {
            resolve_expression_identifiers(&mut if_statement.condition, &working_identifiers);
            resolve_statement_identifiers(&if_statement.statement, &working_identifiers);
            if let Some(ref mut else_statement) = if_statement.else_statement {
                resolve_statement_identifiers(&else_statement, &working_identifiers);
            }
        }
        StatementContents::SwitchStatement(ref mut switch_statement) => {
            resolve_expression_identifiers(
                &mut switch_statement.match_expression,
                &working_identifiers,
            );
            resolve_statement_identifiers(&switch_statement.statement, &working_identifiers);
        }
        StatementContents::WhileLoop(ref mut while_loop) => {
            resolve_expression_identifiers(&mut while_loop.condition, &working_identifiers);
            resolve_statement_identifiers(&while_loop.statement, &working_identifiers);
        }
        StatementContents::ForLoop(ref mut for_loop) => {
            let mut locals = working_identifiers.clone();
            match &for_loop.initializer {
                Node::Declaration(declaration) => {
                    resolve_declaration_identifiers(&declaration, &mut locals)
                }
                Node::Statement(statement) => resolve_statement_identifiers(&statement, &locals),
            }
            resolve_expression_identifiers(&mut for_loop.condition, &locals);
            resolve_expression_identifiers(&mut for_loop.incrementer, &locals);
            resolve_statement_identifiers(&for_loop.statement, &locals);
        }
        StatementContents::GotoStatement(ref mut identifier) => {
            if let Some(declaration_link) = working_identifiers.get(&identifier.hash) {
                identifier.declaration = Some(declaration_link.clone());
            } else {
                panic!("Nonexistant identifier {}", identifier.hash);
            }
        }
        StatementContents::ReturnStatement(ref mut expression) => {
            resolve_expression_identifiers(expression, &working_identifiers)
        }
        _ => {}
    }
}

fn resolve_expression_identifiers(
    expression: &mut Expression,
    working_identifiers: &HashMap<u64, Rc<Declaration>>,
) {
}
