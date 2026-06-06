#![allow(clippy::enum_variant_names)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::too_many_arguments)]

mod grammar_trait {
    include!(concat!(env!("OUT_DIR"), "/grammar_trait.rs"));
}

mod asn1_parser {
    include!(concat!(env!("OUT_DIR"), "/parser.rs"));
}

pub mod grammar;
pub use asn1_parser::parse;
pub use grammar_trait::GrammarTrait;

pub mod ast;
pub mod error;


