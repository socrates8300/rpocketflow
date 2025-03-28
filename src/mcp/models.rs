/// Constants for common Anthropic model names
pub struct Models;

impl Models {
    /// Claude 3 Opus - Most powerful model for complex tasks
    pub const CLAUDE_3_OPUS: &'static str = "claude-3-opus-20240229";
    
    /// Claude 3 Sonnet - Balance of intelligence and speed
    pub const CLAUDE_3_SONNET: &'static str = "claude-3-sonnet-20240229";
    
    /// Claude 3 Haiku - Fastest model for simple tasks
    pub const CLAUDE_3_HAIKU: &'static str = "claude-3-haiku-20240307";
}