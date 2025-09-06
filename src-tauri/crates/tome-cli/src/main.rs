mod clients;
mod db;
mod models;

use anyhow::Context;
use dialoguer::{theme::ColorfulTheme, Select};
use futures_util::StreamExt;
use std::io::Write;

use models::ChatMessage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let conn = match db::connect() {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Error: Could not connect to the Tome database.");
            eprintln!("Please ensure the Tome GUI application has been run at least once.");
            eprintln!("\nDetails: {}", e);
            std::process::exit(1);
        }
    };

    let engines = db::get_engines(&conn)?;

    let mut all_models: Vec<(String, models::Engine)> = Vec::new();

    for engine in engines {
        println!("Fetching models for engine: {}", engine.name);
        let models = match engine.r#type.as_str() {
            "ollama" => {
                let url = engine.options.url.as_deref().unwrap_or("http://localhost:11434");
                clients::ollama::get_models(url).await
            }
            "openai-compat" | "openai" => {
                let url = engine.options.url.as_deref().context("URL not found for OpenAI engine")?;
                let key = engine.options.api_key.as_deref();
                clients::openai::get_models(url, key).await
            }
            _ => {
                println!("  > Engine type '{}' not supported in CLI yet.", engine.r#type);
                Ok(vec![])
            }
        };

        if let Ok(model_names) = models {
            println!("  > Found {} models.", model_names.len());
            for model_name in model_names {
                all_models.push((model_name, engine.clone()));
            }
        } else if let Err(e) = models {
            println!("  > Error fetching models for {}: {}", engine.name, e);
        }
    }

    if all_models.is_empty() {
        println!("No models found from any configured engine.");
        println!("Please ensure Ollama is running and you have pulled a model, e.g., `ollama run llama3`");
        return Ok(());
    }

    let model_display_names: Vec<String> = all_models
        .iter()
        .map(|(name, engine)| format!("{} ({})", name, engine.name))
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a model to chat with:")
        .items(&model_display_names)
        .default(0)
        .interact()?;

    let (selected_model_name, selected_engine) = &all_models[selection];

    println!("Starting chat with {}...", selected_model_name);
    println!("Type 'exit' or 'quit' to end the session.");

    let mut history: Vec<ChatMessage> = Vec::new();

    loop {
        print!("> ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "exit" || input == "quit" {
            break;
        }

        history.push(ChatMessage {
            role: "user".to_string(),
            content: input.to_string(),
        });

        let mut full_response = String::new();
        print!("\nAssistant: ");

        match selected_engine.r#type.as_str() {
            "ollama" => {
                let url = selected_engine.options.url.as_deref().unwrap_or("http://localhost:11434");
                let mut stream =
                    clients::ollama::chat_stream(url, selected_model_name, history.clone()).await?;

                while let Some(Ok(chunk)) = stream.next().await {
                    let content = chunk.message.content;
                    print!("{}", content);
                    std::io::stdout().flush()?;
                    full_response.push_str(&content);

                    if chunk.done {
                        break;
                    }
                }
            }
            "openai-compat" | "openai" => {
                let url = selected_engine.options.url.as_deref().context("URL not found for OpenAI engine")?;
                let key = selected_engine.options.api_key.as_deref();
                let stream =
                    clients::openai::chat_stream(url, selected_model_name, history.clone(), key).await?;
                let mut stream = Box::pin(stream);

                while let Some(Ok(content)) = stream.next().await {
                    print!("{}", content);
                    std::io::stdout().flush()?;
                    full_response.push_str(&content);
                }
            }
            _ => {
                println!("\nChatting with {} engines is not yet implemented in the CLI.", selected_engine.r#type);
                history.pop(); // Remove the user message from history as we didn't process it
                continue;
            }
        }

        history.push(ChatMessage {
            role: "assistant".to_string(),
            content: full_response,
        });
        println!("\n");
    }

    Ok(())
}
