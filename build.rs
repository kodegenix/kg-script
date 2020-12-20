
fn main() {
    kg_syntax_gen::gen_lexer(
        "src/syntax/lexer.g".as_ref(),
        "src/syntax/lexer.rs".as_ref(),
    ).unwrap();
}