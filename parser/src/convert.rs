#![cfg(feature = "convert")]

use crate::*;
use std::path::Path;
use wipple::*;

pub fn convert(ast: &Ast, file: Option<&Path>) -> Value {
    use AstNode::*;

    match &ast.node {
        Block(statements) => Value::of(wipple::Block {
            statements: statements
                .iter()
                .map(|statement| wipple::List {
                    items: statement
                        .items
                        .iter()
                        .map(|node| convert(node, file))
                        .collect(),
                    location: Some(location(&statement.location, file)),
                })
                .collect(),
            location: Some(location(&ast.location, file)),
        }),

        Module(statements) => Value::of(wipple::ModuleBlock {
            statements: statements
                .iter()
                .map(|statement| wipple::List {
                    items: statement
                        .items
                        .iter()
                        .map(|node| convert(node, file))
                        .collect(),
                    location: Some(location(&statement.location, file)),
                })
                .collect(),
            location: Some(location(&ast.location, file)),
        }),

        List(items) => Value::of(wipple::List {
            items: items.iter().map(|node| convert(node, file)).collect(),
            location: Some(location(&ast.location, file)),
        }),

        Quoted(node) => Value::of(wipple::Quoted {
            value: convert(node, file),
            location: Some(location(&ast.location, file)),
        }),

        Name(name) => Value::of(wipple::Name {
            name: name.clone(),
            location: Some(location(&ast.location, file)),
        }),

        Number(number) => Value::of(wipple::Number {
            number: number.clone(),
            location: Some(location(&ast.location, file)),
        }),

        Text(text) => Value::of(wipple::Text {
            text: text.clone(),
            location: Some(location(&ast.location, file)),
        }),
    }
}

fn location(location: &crate::SourceLocation, file: Option<&Path>) -> wipple::SourceLocation {
    wipple::SourceLocation {
        file: file.map(|path| path.to_path_buf()),
        line: location.line,
        column: location.column,
    }
}
