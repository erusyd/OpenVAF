/*
 * ******************************************************************************************
 * Copyright (c) 2019 Pascal Kuthe. This file is part of the VARF project.
 * It is subject to the license terms in the LICENSE file found in the top-level directory
 *  of this distribution and at  https://gitlab.com/jamescoding/VARF/blob/master/LICENSE.
 *  No part of VARF, including this file, may be copied, modified, propagated, or
 *  distributed except according to the terms contained in the LICENSE file.
 * *****************************************************************************************
 */

use sr_alloc::{SliceId, StrId};

use crate::ast::{
    AstAttributeNodeId, AttributeNode, Attributes, Module, ModuleItem, Port, Reference, VerilogType,
};
use crate::error::Error;
use crate::parser::error;
use crate::parser::error::Type::Unsupported;
use crate::parser::error::Unsupported::DefaultDiscipline;
use crate::parser::error::{Expected, Result};
use crate::parser::lexer::Token;
use crate::parser::lexer::Token::{ParenClose, SimpleIdentifier};
use crate::parser::Parser;
use crate::Span;

impl Parser {
    pub(super) fn parse_module(&mut self) -> Result<Module> {
        let start = self.preprocessor.current_start();
        let name = self.parse_identifier(false)?;
        //parameters
        if self.look_ahead()?.0 == Token::Hash {
            self.expect(Token::ParenOpen)?;
            self.parse_parameter_list()?;
            self.expect(Token::ParenClose)?;
        }

        let port_list_start = self.preprocessor.current_start();
        //ports
        let (allow_port_declaration, mut port_list) = if self.look_ahead()?.0 == Token::ParenOpen {
            self.lookahead.take();
            let (next_token, next_span) = self.look_ahead()?;
            let (allow_declarations, ports) = match next_token {
                Token::Input | Token::Output | Token::Inout | Token::ParenOpen => {
                    (false, self.parse_port_declaration_list()?)
                }
                Token::SimpleIdentifier | Token::EscapedIdentifier => {
                    (true, self.parse_port_list()?)
                }
                _ => {
                    return Err(Error {
                        error_type: error::Type::UnexpectedTokens {
                            expected: vec![Expected::PortDeclaration, Expected::Port],
                        },
                        source: next_span,
                    })
                }
            };
            self.expect(Token::ParenClose)?;
            (allow_declarations, ports)
        } else {
            (false, Vec::new())
        };
        let port_list_span = self
            .span_to_current_end(port_list_start)
            .negative_offset(start);

        self.expect(Token::Semicolon)?;

        let mut macro_items = Vec::new();
        loop {
            let (token, span) = self.look_ahead()?;
            match token {
                Token::Inout | Token::Input | Token::Output if allow_port_declaration => {
                    port_list.append(&mut self.parse_port_declaration()?)
                }
                Token::Inout | Token::Input | Token::Output => {
                    let source = self
                        .parse_port_declaration()?
                        .last()
                        .unwrap()
                        .contents
                        .source //we do this here so that the error doesnt just underline the input token but the entire declaration instead
                        .negative_offset(start);
                    return Err(Error {
                        source: self.span_to_current_end(start),
                        error_type: error::Type::PortRedeclaration(source, port_list_span),
                    });
                }
                Token::EOF => {
                    return Err(Error {
                        error_type: error::Type::UnexpectedEof {
                            expected: vec![Token::EndModule],
                        },
                        source: span,
                    })
                }
                Token::EndModule => {
                    self.lookahead.take();
                    break;
                }
                token => macro_items.push(self.parse_module_item()?),
            }
        }
        Ok(Module {
            name,
            port_list: self.ast_allocator.alloc_slice_copy(port_list.as_slice()),
            children: self.ast_allocator.alloc_slice_copy(macro_items.as_slice()),
        })
    }

    fn parse_port_list(&mut self) -> Result<Vec<AttributeNode<Port>>> {
        let name = self.parse_identifier(false)?;
        let mut res = vec![AttributeNode::new(
            self.preprocessor.current_span(),
            Attributes::dangling(),
            Port {
                name,
                ..Port::default()
            },
        )];
        while self.look_ahead()?.0 == Token::Comma {
            self.lookahead.take();
            let name = self.parse_identifier(false)?;
            res.push(AttributeNode::new(
                self.preprocessor.current_span(),
                Attributes::dangling(),
                Port {
                    name,
                    ..Port::default()
                },
            ))
        }
        Ok(res)
    }
    fn parse_port_declaration_list(&mut self) -> Result<Vec<AttributeNode<Port>>> {
        if self.look_ahead()?.0 == ParenClose {
            return Ok(Vec::new());
        }
        let mut start = self.look_ahead()?.1.get_start();
        let mut attributes = self.parse_attributes()?;
        let mut port = self.parse_port_declaration_base()?;
        let mut res = vec![AttributeNode::new(
            self.span_to_current_end(start),
            attributes,
            port,
        )];
        while self.look_ahead()?.0 == Token::Comma {
            while self.next()?.0 == Token::Comma {
                if let Ok(name) = self.parse_identifier(true) {
                    port.name = name;
                    res.push(AttributeNode::new(
                        self.span_to_current_end(start),
                        attributes,
                        port,
                    ));
                } else {
                    break;
                }
            }
            start = self.look_ahead()?.1.get_start();
            attributes = self.parse_attributes()?;
            port = self.parse_port_declaration_base()?;
            res.push(AttributeNode::new(
                self.span_to_current_end(start),
                attributes,
                port,
            ));
        }
        Ok(res)
    }

    /// this parses a port Declaration which only declares one port (for example input electrical x but not input electrical x,y)
    /// this function is a helper function to either be called from parse_port_declaration or parse_port_declaration_list which handel the extra ports declared
    fn parse_port_declaration_base(&mut self) -> Result<Port> {
        let (token, span) = self.next()?;
        let (input, output) = match token {
            Token::Input => (true, false),
            Token::Output => (false, true),
            Token::Inout => (true, true),
            _ => {
                return Err(Error {
                    source: span,
                    error_type: error::Type::UnexpectedToken {
                        expected: vec![Token::Inout, Token::Input, Token::Output],
                    },
                })
            }
        };

        let opt_first_identifier_or_discipline = self.parse_identifier(true);
        let mut is_discipline = false; //helps resolve the ambiguity whether an identifier refers to the first name or the discipline of a port declaration
        let token = self.look_ahead()?.0;
        let port_type = if let Ok(port_type) = self.parse_type(token) {
            self.lookahead.take();
            is_discipline = true;
            port_type
        } else {
            VerilogType::UNDECLARED
        };

        let signed = if self.look_ahead()?.0 == Token::Signed {
            self.lookahead.take();
            is_discipline = true;
            true
        } else {
            false
        };

        let (name, discipline) = match opt_first_identifier_or_discipline {
            Ok(discipline) if is_discipline => (
                self.parse_identifier(false)?,
                Some(Reference::new(discipline)),
            ),
            Ok(first_identifier_or_discipline) => {
                if let Ok(first_identifier) = self.parse_identifier(true) {
                    (
                        first_identifier,
                        Some(Reference::new(first_identifier_or_discipline)),
                    )
                } else {
                    (first_identifier_or_discipline, None)
                }
            }
            Err(_) => (self.parse_identifier(false)?, None),
        }; //TODO default discipline
        Ok(Port {
            name,
            input,
            output,
            discipline,
            signed,
            verilog_type: port_type,
        })
    }

    pub fn parse_port_declaration(&mut self) -> Result<Vec<AttributeNode<Port>>> {
        let start = self.look_ahead()?.1.get_start();
        let port = self.parse_port_declaration_base()?;
        let attributes = self.parse_attributes()?;
        let mut res = vec![AttributeNode::new(
            self.span_to_current_end(start),
            attributes,
            port,
        )];
        while self.look_ahead()?.0 != Token::Semicolon {
            self.expect(Token::Comma);
            let port = Port {
                name: self.parse_identifier(false)?,
                ..port
            };
            res.push(AttributeNode::new(
                self.span_to_current_end(start),
                attributes,
                port,
            ));
        }
        self.lookahead.take();
        Ok(res)
    }

    fn parse_parameter_list(&mut self) -> Result {
        unimplemented!()
    }
    fn parse_module_item(&mut self) -> Result<AttributeNode<ModuleItem>> {
        let attributes = self.parse_attributes()?;
        let start = self.look_ahead()?.1.get_start();
        let contents = match self.look_ahead()?.0 {
            Token::Analog => {
                self.lookahead.take();
                unimplemented!("Analog_Block")
            }
            Token::Branch => {
                self.lookahead.take();
                let branch_decl = self.parse_branch_declaration()?;
                ModuleItem::BranchDecl(branch_decl)
            }
            _ => unimplemented!("Variable Declaration"),
        };
        let span = self.span_to_current_end(start);
        Ok(AttributeNode::new(span, attributes, contents))
    }
}
