use std::{fmt::{Debug, Display, Write}, sync::Arc};

use cranelift::codegen::CodegenError;

use crate::value::JValue;



#[derive(Clone)]
pub enum Error{
    IllegalBreakStatement,
    IllegalContinueStatment,

    UndefinedLabel(String),

    Break(Option<String>),
    Continue(Option<String>),
    Return(JValue),

    Deprecated(&'static str),
    Unimplemented(&'static str),

    CodegenError(Arc<CodegenError>),

    ParseError(swc_ecma_parser::error::Error),

    Value(JValue),
}

impl Error{

}

impl std::error::Error for Error{

}

impl Debug for Error{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Error{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            Error::Break(_) => f.write_str("Illegal break statment."),
            Error::Continue(_) => f.write_str("Illegal continue statment."),
            Error::Return(_) => f.write_str("Illegal return statment."),
            Error::UndefinedLabel(l) => f.write_fmt(format_args!("Undefined label: {}.", l)),
            Error::IllegalBreakStatement => f.write_str("Illegal break statment."),
            Error::IllegalContinueStatment => f.write_str("Illegal continue statment."),
            Error::Deprecated(s) => f.write_fmt(format_args!("Deprecated: {}", s)),
            Error::Unimplemented(s) => f.write_fmt(format_args!("Unimplemented: {}", s)),
            Error::CodegenError(c) => Display::fmt(c, f),
            Error::ParseError(p) => Display::fmt(&p.kind().msg(), f),
            Error::Value(v) => f.write_str(v.to_string().as_str()),
        }
    }
}