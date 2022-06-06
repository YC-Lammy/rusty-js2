
use swc_ecma_parser::{
    Parser,
    Syntax,
    EsConfig,
    TsConfig
};
use swc_ecma_ast::Module;
use swc_common::{input::StringInput, BytePos};

use crate::error::Error;

pub fn parse_ecma(filename:&str, script:&str) -> Result<Module, Error>{

    let input = StringInput::new(script, BytePos(0), BytePos(script.len() as u32));
    let mut parser = Parser::new(
        swc_ecma_parser::Syntax::Es(EsConfig{
            jsx:false,
            fn_bind:true,
            decorators:true,
            decorators_before_export:true,
            export_default_from:true,
            import_assertions:true,
            static_blocks:true,
            private_in_object:true,
        }), 
        input, None);

    let re = parser.parse_module();

    match re{
        Ok(v) => Ok(v),
        Err(e) => Err(Error::ParseError(e))
    }
}