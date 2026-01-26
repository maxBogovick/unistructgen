use unistructgen_markdown_parser::chunker::{SemanticChunker, SplitterOptions};

#[test]
fn test_semantic_splitting_simple() {
    let markdown = r#"
# Introduction
This is the intro.

# Chapter 1
## Section A
This is section A content.

## Section B
This is section B content.
"#;

    let splitter = SemanticChunker::new(SplitterOptions::default());
    let chunks = splitter.split(markdown);

    // Expected:
    // 1. Intro
    // 2. Section A (under Chapter 1)
    // 3. Section B (under Chapter 1)
    
    assert_eq!(chunks.len(), 3);
    
    // Chunk 1
    assert_eq!(chunks[0].header_path, vec!["Introduction"]);
    assert_eq!(chunks[0].content, "This is the intro.");
    
    // Chunk 2
    assert_eq!(chunks[1].header_path, vec!["Chapter 1", "Section A"]);
    assert_eq!(chunks[1].content, "This is section A content.");

    // Chunk 3
    assert_eq!(chunks[2].header_path, vec!["Chapter 1", "Section B"]);
    assert_eq!(chunks[2].content, "This is section B content.");
}

#[test]
fn test_code_blocks() {
    let markdown = r#"
# Coding
Here is some code:

```rust
fn main() {
    println!("Hello");
}
```

End of code.
"#;
    let splitter = SemanticChunker::new(SplitterOptions::default());
    let chunks = splitter.split(markdown);
    
    // Expect:
    // 1. "Here is some code:"
    // 2. Code block
    // 3. "End of code."
    
    assert_eq!(chunks.len(), 3);
    
    assert!(chunks[1].metadata.is_code);
    assert_eq!(chunks[1].metadata.language.as_deref(), Some("rust"));
    assert!(chunks[1].content.contains("println!"));
}
