use pulldown_cmark::{Event, Parser, Tag, HeadingLevel};
use serde::{Deserialize, Serialize};

/// A semantic chunk of text from a Markdown document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarkdownChunk {
    /// The actual text content of the chunk.
    pub content: String,
    /// The hierarchy of headers leading to this chunk (e.g., ["Chapter 1", "Section A"])
    pub header_path: Vec<String>,
    /// The starting byte offset in the original document (approximate).
    pub offset: usize,
    /// Metadata tags (e.g., if it's a code block, or language).
    pub metadata: ChunkMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ChunkMetadata {
    pub is_code: bool,
    pub language: Option<String>,
    pub is_table: bool,
}

/// Options for splitting markdown.
#[derive(Debug, Clone)]
pub struct SplitterOptions {
    /// Maximum characters per chunk (soft limit, tries to break at semantic boundaries).
    pub max_chunk_size: usize,
    /// Overlap between chunks (not yet implemented for semantic splitter, but reserved).
    pub overlap: usize,
    /// If true, code blocks are kept as single chunks even if they exceed max size.
    pub keep_code_blocks_atomic: bool,
}

impl Default for SplitterOptions {
    fn default() -> Self {
        Self {
            max_chunk_size: 1000,
            overlap: 0,
            keep_code_blocks_atomic: true,
        }
    }
}

#[allow(dead_code)]
pub struct SemanticChunker {
    options: SplitterOptions,
}

impl SemanticChunker {
    pub fn new(options: SplitterOptions) -> Self {
        Self { options }
    }

    pub fn split(&self, markdown: &str) -> Vec<MarkdownChunk> {
        let parser = Parser::new(markdown);
        
        let mut chunks = Vec::new();
        let mut header_stack: Vec<(HeadingLevel, String)> = Vec::new();
        
        let mut current_buffer = String::new();
        let mut current_metadata = ChunkMetadata::default();
        
        let mut in_heading = false;
        let mut heading_buffer = String::new();
        
        for event in parser {
            match event {
                Event::Start(Tag::Heading(level, _, _)) => {
                    // If we have accumulated content, save it as a chunk attached to *previous* context
                    if !current_buffer.trim().is_empty() {
                         chunks.push(MarkdownChunk {
                            content: current_buffer.trim().to_string(),
                            header_path: header_stack.iter().map(|(_, t)| t.clone()).collect(),
                            offset: 0, 
                            metadata: current_metadata.clone(),
                        });
                        current_buffer.clear();
                        current_metadata = ChunkMetadata::default();
                    }
                    
                    in_heading = true;
                    
                    // Adjust stack for new level
                    while let Some((last_level, _)) = header_stack.last() {
                        if *last_level >= level {
                            header_stack.pop();
                        } else {
                            break;
                        }
                    }
                    // Push temporary placeholder
                    header_stack.push((level, String::new()));
                }
                Event::End(Tag::Heading(_, _, _)) => {
                    in_heading = false;
                    // Update the title of the current header in stack
                    if let Some((_, title)) = header_stack.last_mut() {
                        *title = heading_buffer.trim().to_string();
                    }
                    heading_buffer.clear();
                }
                Event::Text(text) => {
                    if in_heading {
                        heading_buffer.push_str(&text);
                    } else {
                        current_buffer.push_str(&text);
                    }
                }
                Event::Code(text) => {
                    if in_heading {
                        heading_buffer.push_str("`");
                        heading_buffer.push_str(&text);
                        heading_buffer.push_str("`");
                    } else {
                        current_buffer.push_str("`");
                        current_buffer.push_str(&text);
                        current_buffer.push_str("`");
                    }
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    // Flush preceding text
                     if !current_buffer.trim().is_empty() {
                         chunks.push(MarkdownChunk {
                            content: current_buffer.trim().to_string(),
                            header_path: header_stack.iter().map(|(_, t)| t.clone()).collect(),
                            offset: 0, 
                            metadata: current_metadata.clone(),
                        });
                        current_buffer.clear();
                        current_metadata = ChunkMetadata::default();
                    }
                    
                    current_metadata.is_code = true;
                    if let pulldown_cmark::CodeBlockKind::Fenced(lang) = kind {
                         current_metadata.language = Some(lang.to_string());
                    }
                    
                    // Reconstruct code block start
                    current_buffer.push_str("```");
                    if let Some(lang) = &current_metadata.language {
                        current_buffer.push_str(lang);
                    }
                    current_buffer.push('\n');
                }
                Event::End(Tag::CodeBlock(_)) => {
                    current_buffer.push_str("\n```\n");
                    
                    // Flush the code block immediately
                    chunks.push(MarkdownChunk {
                        content: current_buffer.trim().to_string(),
                        header_path: header_stack.iter().map(|(_, t)| t.clone()).collect(),
                        offset: 0, 
                        metadata: current_metadata.clone(),
                    });
                    current_buffer.clear();
                    current_metadata = ChunkMetadata::default();
                }
                Event::SoftBreak => {
                    if !in_heading { current_buffer.push('\n'); }
                }
                Event::HardBreak => {
                    if !in_heading { current_buffer.push_str("\n\n"); }
                }
                Event::Start(Tag::Paragraph) => {}
                Event::End(Tag::Paragraph) => {
                     if !in_heading { current_buffer.push_str("\n\n"); }
                }
                 Event::Start(Tag::List(_)) => {}
                 Event::End(Tag::List(_)) => {
                      if !in_heading { current_buffer.push_str("\n"); }
                 }
                 Event::Start(Tag::Item) => {
                      if !in_heading { current_buffer.push_str("- "); }
                 }
                 Event::End(Tag::Item) => {
                      if !in_heading { current_buffer.push('\n'); }
                 }
                _ => {}
            }
        }
        
        // Final flush
        if !current_buffer.trim().is_empty() {
             chunks.push(MarkdownChunk {
                content: current_buffer.trim().to_string(),
                header_path: header_stack.iter().map(|(_, t)| t.clone()).collect(),
                offset: 0, 
                metadata: current_metadata.clone(),
            });
        }
        
        chunks
    }
}
