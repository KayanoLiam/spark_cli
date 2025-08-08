use spark_cli::utils::code::{extract_code_blocks, choose_best_block};

#[test]
fn test_extract_single_block() {
    let s = "here\n```rust\nfn main(){}\n```\nend";
    let blocks = extract_code_blocks(s);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].language.as_deref(), Some("rust"));
}

#[test]
fn test_choose_best_by_lang() {
    let s = "```python\nprint('hi')\n```\n```rust\nfn main(){}\n```";
    let blocks = extract_code_blocks(s);
    let best = choose_best_block(&blocks, &["rust"]).unwrap();
    assert!(best.content.contains("fn main"));
}
