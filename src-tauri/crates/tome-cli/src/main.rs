mod clients;
mod db;
mod models;

use dialoguer::{theme::ColorfulTheme, Select};
use models::Engine;
use std::io::Write;

use models::ClientOptions;

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

    let mut all_models: Vec<(String, String, String)> = Vec::new(); // (model_name, engine_name, engine_url)

    for engine in &engines {
        if engine.r#type == "ollama" {
            if let Some(url) = &engine.options.url {
                if let Ok(models) = clients::ollama::get_models(url).await {
                    for model_name in models {
                        all_models.push((model_name, engine.name.clone(), url.clone()));
                    }
                }
            }
        }
    }

    if all_models.is_empty() {
        println!("No models found from any configured engine.");
        println!("Please ensure Ollama is running and you have pulled a model, e.g., `ollama run llama3`");
        return Ok(());
    }

    let model_names: Vec<String> = all_models
        .iter()
        .map(|(name, engine_name, _)| format!("{} ({})", name, engine_name))
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a model to chat with:")
        .items(&model_names)
        .default(0)
        .interact()?;

    let (selected_model, _, ollama_url) = &all_models[selection];

    println!("Starting chat with {}...", selected_model);
    println!("Type 'exit' or 'quit' to end the session.");

    let mut history: Vec<clients::ollama::ChatMessage> = Vec::new();

    loop {
        print!("> ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "exit" || input == "quit" {
            break;
        }

        history.push(clients::ollama::ChatMessage {
            role: "user".to_string(),
            content: input.to_string(),
        });

        let mut stream =
            clients::ollama::chat_stream(ollama_url, selected_model, history.clone()).await?;

        let mut full_response = String::new();
        print!("\nAssistant: ");
        while let Some(Ok(chunk)) = stream.next().await {
            let content = chunk.message.content;
            print!("{}", content);
            std::io::stdout().flush()?;
            full_response.push_str(&content);

            if chunk.done {
                break;
            }
        }

        history.push(clients::ollama::ChatMessage {
            role: "assistant".to_string(),
            content: full_response,
        });
        println!("\n");
    }

    Ok(())
}
