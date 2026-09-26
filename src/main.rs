mod tools;

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionMessageToolCalls, ChatCompletionRequestMessage,
        ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
        ChatCompletionTool, ChatCompletionTools, CreateChatCompletionRequest, FunctionObject,
    },
};
use clap::Parser;
use serde_json::json;
use std::{env, process};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let base_url = env::var("OPENROUTER_BASE_URL")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

    let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        eprintln!("OPENROUTER_API_KEY is not set");
        process::exit(1);
    });

    let env = env::var("ENV")
        .unwrap_or_else(|_| String::from("dev"))
        .to_lowercase();
    let model = if env == "local" {
        "stealth/space-bunny-alpha"
    } else {
        "anthropic/claude-haiku-4.5"
    };

    let config = OpenAIConfig::new()
        .with_api_base(base_url)
        .with_api_key(api_key);

    let client = Client::with_config(config);

    let request = CreateChatCompletionRequest {
        model: model.to_string(),
        messages: vec![ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text(args.prompt.clone()),
                name: None,
            },
        )],
        tools: Some(vec![ChatCompletionTools::Function(ChatCompletionTool {
            function: FunctionObject {
                name: "read".to_string(),
                description: Some("Read and return the contents of a file".to_string()),
                parameters: Some(json!({
                      "type": "object",
                      "properties": {
                        "file_path": {
                          "type": "string",
                          "description": "The path to the file to read"
                        }
                      },
                      "required": ["file_path"]

                })),
                strict: None,
            },
        })]),
        ..Default::default()
    };

    let response = client.chat().create(request).await?;

    let chat_choice = response
        .choices
        .first()
        .expect("Expected one choice in reponse");

    if let Some(content) = &chat_choice.message.content {
        println!("{}", content)
    };

    for tool_call in chat_choice.message.tool_calls.iter().flatten() {
        match tool_call {
            ChatCompletionMessageToolCalls::Function(call) => {
                tools::call_tool(&call.function.name, &call.function.arguments)?;
            }
            _ => (),
        }
    }

    Ok(())
}
