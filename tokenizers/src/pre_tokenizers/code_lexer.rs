use serde::{Deserialize, Deserializer, Serialize};

use crate::tokenizer::normalizer::Range;
use crate::tokenizer::{PreTokenizedString, PreTokenizer, Result};
use crate::utils::SysRegex;

/// CodeLexer pre-tokenizer that applies language-specific lexing to code blocks.
///
/// This pre-tokenizer:
/// 1. Finds code fence blocks (```language\n...\n```)
/// 2. Applies language-specific lexers to extract tokens
/// 3. Leaves non-code text unchanged
///
/// Currently supported languages:
/// - Python (using rustpython-parser lexer)
///
/// Other languages will print a warning and return the original code block.
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub struct CodeLexer {
    /// List of language identifiers to apply lexing to (e.g., ["python", "py"])
    pub languages: Vec<String>,
}

impl<'de> Deserialize<'de> for CodeLexer {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        enum Type {
            CodeLexer,
        }

        #[derive(Deserialize)]
        pub struct CodeLexerHelper {
            #[serde(rename = "type")]
            _type: Type,
            #[serde(default = "default_languages")]
            languages: Vec<String>,
        }

        let helper = CodeLexerHelper::deserialize(deserializer)?;
        Ok(Self::new(helper.languages))
    }
}

fn default_languages() -> Vec<String> {
    vec![
        "python".to_string(),
        "py".to_string(),
    ]
}

// impl Clone for CodeLexer {
//     fn clone(&self) -> Self {
//         Self::new(self.languages.clone())
//     }
// }

impl PartialEq for CodeLexer {
    fn eq(&self, other: &Self) -> bool {
        self.languages == other.languages
    }
}

impl CodeLexer {
    pub fn new(languages: Vec<String>) -> Self {
        Self { languages }
    }

    pub fn default() -> Self {
        Self::new(default_languages())
    }

    /// Check if a language is supported for lexing
    fn is_supported_language(&self, lang: &str) -> bool {
        self.languages.iter().any(|l| l.eq_ignore_ascii_case(lang))
    }

    /// Apply Python lexer to code string
    #[cfg(feature = "python_lexer")]
    fn lex_python(&self, code: &str, offset: usize) -> Result<Vec<(usize, usize)>> {
        use rustpython_parser::lexer::lex;
        use rustpython_parser::Mode;
        use rustpython_parser::Tok;

        let mut boundaries: Vec<(usize, usize)> = Vec::new();

        // Lex the Python code
        let tokens = lex(code, Mode::Module);

        for token_result in tokens {
            match token_result {
                Ok((token, range)) => {
                    // Convert TextRange to byte offsets
                    let start = range.start().to_usize() + offset;
                    let end = range.end().to_usize() + offset;

                    // Skip ENDMARKER token
                    if matches!(token, Tok::EndOfFile) {
                        continue;
                    }

                    // For newline tokens, attach to previous token instead of creating separate token
                    if matches!(token, Tok::Newline) {
                        if let Some(last_boundary) = boundaries.last_mut() {
                            // Extend previous token's end by 1 to include the newline
                            last_boundary.1 += 1;
                            // Don't add the newline as a separate boundary
                            continue;
                        }
                    }

                    boundaries.push((start, end));
                }
                Err(e) => {
                    // Log error but don't fail - return original text
                    eprintln!("Warning: Failed to lex Python code: {:?}", e);
                    return Ok(vec![(offset, offset + code.len())]);
                }
            }
        }

        // Fill in whitespace gaps by extending the next token backwards
        let mut filled_boundaries = Vec::new();
        let mut last_end = offset;

        for (start, end) in boundaries {
            // If there's a gap (whitespace), attach it to the current token by moving start backwards
            let adjusted_start = if start > last_end {
                last_end  // Extend current token to include whitespace before it
            } else {
                start
            };
            filled_boundaries.push((adjusted_start, end));
            last_end = end;
        }

        Ok(filled_boundaries)
    }

    #[cfg(not(feature = "python_lexer"))]
    fn lex_python(&self, code: &str, offset: usize) -> Result<Vec<(usize, usize)>> {
        eprintln!("Warning: Python lexer not available (feature 'python_lexer' not enabled)");
        Ok(vec![(offset, offset + code.len())])
    }

    /// Apply language-specific lexer
    fn lex_code(&self, lang: &str, code: &str, offset: usize) -> Result<Vec<(usize, usize)>> {
        let lang_lower = lang.to_lowercase();

        if lang_lower == "python" || lang_lower == "py" {
            self.lex_python(code, offset)
        } else if self.is_supported_language(&lang_lower) {
            // Language is in the list but not implemented
            eprintln!("Warning: Lexer for '{}' not implemented yet. Returning original code block.", lang);
            Ok(vec![(offset, offset + code.len())])
        } else {
            // Unknown language, return as-is
            Ok(vec![(offset, offset + code.len())])
        }
    }
}

impl PreTokenizer for CodeLexer {
    fn pre_tokenize(&self, pretokenized: &mut PreTokenizedString) -> Result<()> {
        eprintln!("[CodeLexer] pre_tokenize called with languages: {:?}", self.languages);

        // Regex to find code fence blocks: ```language\ncode\n```
        // Use [\s\S] instead of . to match any character including newlines (Oniguruma compatible)
        let code_fence_regex = SysRegex::new(r"```(\w+)?\n([\s\S]*?)```")?;

        pretokenized.split(|_idx, normalized| {
            let text = normalized.get();
            // eprintln!("[CodeLexer] Processing text of length: {}", text.len());

            // Find all code blocks
            let mut last_end = 0;
            let mut splits = Vec::new();

            for (match_start, match_end) in code_fence_regex.find_iter(text) {
                // eprintln!("[CodeLexer] Found code fence at {}..{}", match_start, match_end);

                // Add text before code block
                if match_start > last_end {
                    splits.push((last_end, match_start));
                }

                // Try to extract language and code from the match
                // Pattern: ```lang\ncode```
                let match_text = &text[match_start..match_end];

                // Simple parsing: find first newline after ``` to get language
                if let Some(first_newline) = match_text.find('\n') {
                    let fence_start_end = match_start + first_newline + 1;

                    // Find closing ```
                    if let Some(closing_fence_start) = match_text.rfind("```") {
                        let closing_fence_abs = match_start + closing_fence_start;

                        // Extract language (between ``` and \n)
                        let lang_part = &match_text[3..first_newline];

                        // Add opening fence (```lang\n)
                        splits.push((match_start, fence_start_end));

                        // Extract and lex code block
                        let code = &text[fence_start_end..closing_fence_abs];

                        if !lang_part.is_empty() && self.is_supported_language(lang_part) {
                            // eprintln!("[CodeLexer] Applying lexer for language: '{}'", lang_part);
                            // Apply language-specific lexing
                            match self.lex_code(lang_part, code, fence_start_end) {
                                Ok(boundaries) => {
                                    // eprintln!("[CodeLexer] Lexed {} tokens from {} code", boundaries.len(), lang_part);
                                    // Add lexed boundaries
                                    for (start, end) in boundaries {
                                        splits.push((start, end));
                                    }
                                }
                                Err(_) => {
                                    // Fallback: add code as single block
                                    splits.push((fence_start_end, closing_fence_abs));
                                }
                            }
                        } else {
                            // Unknown or unsupported language - add as single block
                            splits.push((fence_start_end, closing_fence_abs));
                        }

                        // Add closing fence (```)
                        splits.push((closing_fence_abs, match_end));
                    }
                }

                last_end = match_end;
            }

            // Add remaining text
            if last_end < text.len() {
                splits.push((last_end, text.len()));
            }

            // If no code blocks found, return original
            if splits.is_empty() {
                splits.push((0, text.len()));
            }

            // Add a debug print of the splits
            // eprintln!("[CodeLexer] Splits: {:?}", splits);

            // Convert splits to normalized slices
            Ok(splits
                .into_iter()
                .filter_map(|(start, end)| {
                    if start < end {
                        normalized.slice(Range::Original(start..end))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{OffsetReferential, OffsetType};

    #[test]
    fn no_code_blocks() {
        let text = "This is just plain text without code.";
        let mut pretokenized = PreTokenizedString::from(text);
        let lexer = CodeLexer::default();

        lexer.pre_tokenize(&mut pretokenized).unwrap();

        let splits = pretokenized.get_splits(OffsetReferential::Original, OffsetType::Byte);
        assert_eq!(splits.len(), 1);
        assert_eq!(splits[0].0, text);
    }

    #[test]
    fn simple_code_block() {
        let text = "```python\ndef test(): pass\n```";
        let mut pretokenized = PreTokenizedString::from(text);
        let lexer = CodeLexer::default();

        lexer.pre_tokenize(&mut pretokenized).unwrap();

        let splits = pretokenized.get_splits(OffsetReferential::Original, OffsetType::Byte);
        // Should have at least: opening fence, code tokens, closing fence
        assert!(splits.len() >= 3);
    }

    #[test]
    fn serialization() {
        let lexer = CodeLexer::default();
        let lexer_s = r#"{"type":"CodeLexer","languages":["python","py"]}"#;

        assert_eq!(serde_json::to_string(&lexer).unwrap(), lexer_s);
        assert_eq!(serde_json::from_str::<CodeLexer>(lexer_s).unwrap(), lexer);
    }

    #[test]
    fn mixed_content() {
        let text = "Here is code:\n```python\nx = 1\n```\nMore text.";
        let mut pretokenized = PreTokenizedString::from(text);
        let lexer = CodeLexer::default();

        lexer.pre_tokenize(&mut pretokenized).unwrap();

        // Should split into: before, fence, code, fence, after
        let splits = pretokenized.get_splits(OffsetReferential::Original, OffsetType::Byte);
        assert!(splits.len() >= 4);
    }
}
