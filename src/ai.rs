use anyhow::Result;
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
};
use async_openai::{config::OpenAIConfig, Client as OpenAIClient};
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use toml;

#[derive(serde::Deserialize, Default)]
pub struct AiConfig {
    pub openai_api_key: Option<String>,
    pub model: Option<String>,
}

pub fn read_ai_config(devlog_path: &PathBuf) -> io::Result<AiConfig> {
    let cfg_path = devlog_path.join("config.toml");
    if cfg_path.exists() {
        let mut s = String::new();
        File::open(cfg_path)?.read_to_string(&mut s)?;
        let cfg: AiConfig = toml::from_str(&s).unwrap_or_default();
        Ok(cfg)
    } else {
        Ok(AiConfig::default())
    }
}

pub fn load_devlog_context(devlog_path: &PathBuf, max_bytes: usize) -> io::Result<String> {
    let files = list_existing_devlog_files(devlog_path)?;
    let mut acc = String::new();
    
    for (idx, fname) in files.iter().enumerate() {
        let path = devlog_path.join(fname);
        if path.is_file() {
            let mut content = String::new();
            if let Ok(mut f) = File::open(&path) {
                let _ = f.read_to_string(&mut content);
            }
            acc.push_str(&format!("\n\n# File {}: {}\n\n{}\n", idx + 1, fname, content));
            if acc.len() >= max_bytes {
                break;
            }
        }
    }
    Ok(acc)
}

pub async fn ask_question(
    client: &OpenAIClient<OpenAIConfig>,
    model: &str,
    context: &str,
    question: &str,
) -> Result<String> {
    let system_msg: ChatCompletionRequestMessage = ChatCompletionRequestSystemMessageArgs::default()
        .content(context)
        .build()?
        .into();

    let user_msg: ChatCompletionRequestMessage = ChatCompletionRequestUserMessageArgs::default()
        .content(question)
        .build()?
        .into();

    let req = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages([system_msg, user_msg])
        .build()?;

    let response = client.chat().create(req).await?;
    Ok(response
        .choices
        .first()
        .and_then(|c| c.message.content.clone())
        .unwrap_or_default())
}

pub fn create_client(api_key: &str) -> OpenAIClient<OpenAIConfig> {
    let config = OpenAIConfig::new().with_api_key(api_key.to_string());
    OpenAIClient::with_config(config)
}

// Helper function to list devlog files (moved from main.rs)
pub fn list_existing_devlog_files(devlog_path: &PathBuf) -> io::Result<Vec<String>> {
    let mut files = Vec::new();
    if devlog_path.exists() {
        for entry in std::fs::read_dir(devlog_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "md" || ext == "markdown" {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            files.push(name.to_string());
                        }
                    }
                }
            }
        }
        files.sort();
    }
    Ok(files)
}
