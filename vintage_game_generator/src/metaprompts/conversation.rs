use serde::{Deserialize, Serialize};
use async_openai::types::ChatCompletionRequestMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationState {
    pub messages: Vec<ChatCompletionRequestMessage>,
    pub start_time: String,
    pub current_phase: String,
}

impl ConversationState {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            start_time: chrono::Utc::now().to_rfc3339(),
            current_phase: "design".to_string(),
        }
    }
    
    pub fn get_history(&self) -> Vec<ConversationMessage> {
        self.messages.iter().map(|msg| {
            let (role, content) = match msg {
                ChatCompletionRequestMessage::System(m) => {
                    ("system", match &m.content {
                        async_openai::types::ChatCompletionRequestSystemMessageContent::Text(text) => text.clone(),
                        async_openai::types::ChatCompletionRequestSystemMessageContent::Array(_) => "[complex content]".to_string(),
                    })
                },
                ChatCompletionRequestMessage::User(m) => {
                    ("user", match &m.content {
                        async_openai::types::ChatCompletionRequestUserMessageContent::Text(text) => text.clone(),
                        async_openai::types::ChatCompletionRequestUserMessageContent::Array(_) => "[complex content]".to_string(),
                    })
                },
                ChatCompletionRequestMessage::Assistant(m) => {
                    ("assistant", m.content.as_ref().map(|c| match c {
                        async_openai::types::ChatCompletionRequestAssistantMessageContent::Text(text) => text.clone(),
                        async_openai::types::ChatCompletionRequestAssistantMessageContent::Array(_) => "[complex content]".to_string(),
                    }).unwrap_or_default())
                },
                _ => ("unknown", String::new()),
            };
            
            ConversationMessage {
                role: role.to_string(),
                content,
                timestamp: chrono::Utc::now().to_rfc3339(),
            }
        }).collect()
    }
    
    pub fn to_markdown(&self) -> String {
        let mut markdown = String::from("# Game Design Conversation\n\n");
        markdown.push_str(&format!("Started: {}\n\n", self.start_time));
        
        for msg in &self.messages {
            match msg {
                ChatCompletionRequestMessage::User(m) => {
                    let content_str = match &m.content {
                        async_openai::types::ChatCompletionRequestUserMessageContent::Text(text) => text.clone(),
                        async_openai::types::ChatCompletionRequestUserMessageContent::Array(_) => "[complex content]".to_string(),
                    };
                    markdown.push_str(&format!("## User\n{}\n\n", content_str));
                }
                ChatCompletionRequestMessage::Assistant(m) => {
                    if let Some(content) = &m.content {
                        let content_str = match content {
                            async_openai::types::ChatCompletionRequestAssistantMessageContent::Text(text) => text.clone(),
                            async_openai::types::ChatCompletionRequestAssistantMessageContent::Array(_) => "[complex content]".to_string(),
                        };
                        markdown.push_str(&format!("## AI Assistant\n{}\n\n", content_str));
                    }
                }
                _ => {}
            }
        }
        
        markdown
    }
    
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.messages).unwrap_or_default()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

impl ConversationMessage {
    pub fn new(role: &str, content: String) -> Self {
        Self {
            role: role.to_string(),
            content,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}
