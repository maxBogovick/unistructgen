use serde::{Deserialize, Serialize};

/// A proposed fix for a file.
/// This structure is JSON-serializable so we can ask the LLM to output it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFix {
    /// The file to modify (relative path).
    pub file_path: String,
    /// Explanation of why this fix is needed.
    pub explanation: String,
    /// The list of hunks (changes) to apply.
    pub changes: Vec<Hunk>,
}

/// A specific change block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hunk {
    /// The original code to replace (helps in verification and finding the location).
    pub original_snippet: String,
    /// The new code to insert.
    pub new_snippet: String,
    /// Approximate line number to start search (optional hint).
    pub line_hint: Option<usize>,
}

impl CodeFix {
    /// Applies the fix to the file content in memory.
    /// Returns the new content or an error if the original snippet couldn't be found.
    pub fn apply(&self, file_content: &str) -> Result<String, String> {
        let mut new_content = file_content.to_string();
        
        for hunk in &self.changes {
            // Normalize line endings for search
            // For MVP, simplistic string replacement.
            // In production, this needs fuzzy matching or line-based patching.
            
            if let Some(start) = new_content.find(&hunk.original_snippet) {
                new_content.replace_range(
                    start..start + hunk.original_snippet.len(), 
                    &hunk.new_snippet
                );
            } else {
                // Try ignoring whitespace?
                return Err(format!(
                    "Could not find original snippet in {}:\n'{}'", 
                    self.file_path, hunk.original_snippet
                ));
            }
        }
        
        Ok(new_content)
    }
}
