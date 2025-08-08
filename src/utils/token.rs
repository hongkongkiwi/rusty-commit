use anyhow::Result;
use tiktoken_rs::cl100k_base;

/// Estimates the number of tokens in the given text using the OpenAI tokenizer.
/// This uses the cl100k encoding which is used by GPT-3.5 and GPT-4.
///
/// # Examples
///
/// ```ignore
/// use rustycommit::utils::token::estimate_tokens;
///
/// let text = "Hello, world!";
/// let tokens = estimate_tokens(text).unwrap();
/// assert!(tokens > 0);
/// ```
pub fn estimate_tokens(text: &str) -> Result<usize> {
    let bpe = cl100k_base()?;
    let tokens = bpe.encode_with_special_tokens(text);
    Ok(tokens.len())
}
