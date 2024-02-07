use serde::Deserialize;

use std::collections::HashMap;

use crate::bits::Bits;

#[derive(Deserialize, Debug)]
pub struct File {
    pub meta: Meta,
    pub instructions: Instructions,
}

#[derive(Deserialize, Debug)]
pub struct Meta {
    pub name: String,
    pub description: String,
    pub extends: Option<String>,
    pub types: Option<HashMap<String, Type>>,
    pub formats: HashMap<String, Format>,
}

#[derive(Deserialize, Debug)]
pub struct Instructions {
    pub encoding: HashMap<String, FormatEncoding>,
    pub asm: HashMap<String, Assembly>
}

#[derive(Deserialize, Debug)]
pub struct Type {
    pub max_bits: Option<u32>,
    pub enumerate: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct Format {
    pub encoding: Vec<EncodingPart>,
    pub subfields: Option<HashMap<String, Vec<FieldOrBits>>>,
}

#[derive(Deserialize, Debug)]
pub struct EncodingPart {
    pub name: String,
    pub bits: u32,
    pub ty: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum FieldOrBits {
    Bits(Bits),
    Field(String),
}

#[derive(Deserialize, Debug, Clone)]
pub struct FormatEncoding {
    pub format: String,
    #[serde(flatten)]
    pub fields: HashMap<String, FieldOrBits>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Assembly {
    pub mnemonic: String,
    pub fields: Vec<String>,
}

#[test]
fn parse() {
    let content = include_str!("../../format.toml");
    let file: File = toml::from_str(content).unwrap();
    dbg!(file);
    assert!(false);
}