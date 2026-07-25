//! Interactive provider setup for custom LLM endpoints (OpenRouter, Anthropic, OpenAI, DeepSeek, Ollama, etc.).

use anyhow::Result;
use std::io::{self, BufRead, Write};
use xai_grok_tools::util::grok_home::grok_home;

pub fn run_provider_setup() -> Result<()> {
    println!("\n  ┌────────────────────────────────────────────────────────┐");
    println!("  │  Anime / Anibuild — AI Provider Setup                  │");
    println!("  └────────────────────────────────────────────────────────┘\n");
    println!("  Select a provider to configure:\n");
    println!("    1) OpenRouter (Access 100+ models: Claude 3.5, DeepSeek R1, Llama 3, Gemini)");
    println!("    2) Anthropic (Claude 3.5 Sonnet, Claude Opus)");
    println!("    3) OpenAI (GPT-4o, o1, o3-mini)");
    println!("    4) DeepSeek Direct (DeepSeek V3, DeepSeek R1)");
    println!("    5) Ollama (Local models on http://localhost:11434)");
    println!("    6) Custom OpenAI-compatible endpoint\n");
    println!("    q) Quit\n");

    let stdin = io::stdin();
    let mut reader = stdin.lock();

    print!("  Select choice [1-6]: ");
    io::stdout().flush()?;
    let mut choice = String::new();
    reader.read_line(&mut choice)?;
    let choice = choice.trim();

    if choice.eq_ignore_ascii_case("q") {
        println!("Cancelled.");
        return Ok(());
    }

    let (key_name, default_model_id, base_url, default_api_backend, need_api_key, extra_header_key) = match choice {
        "1" => (
            "openrouter-claude",
            "anthropic/claude-3.5-sonnet",
            "https://openrouter.ai/api/v1",
            "chat_completions",
            true,
            None,
        ),
        "2" => (
            "claude-sonnet",
            "claude-3-5-sonnet-20241022",
            "https://api.anthropic.com/v1",
            "messages",
            true,
            Some("x-api-key"),
        ),
        "3" => (
            "gpt-4o",
            "gpt-4o",
            "https://api.openai.com/v1",
            "chat_completions",
            true,
            None,
        ),
        "4" => (
            "deepseek-chat",
            "deepseek-chat",
            "https://api.deepseek.com/v1",
            "chat_completions",
            true,
            None,
        ),
        "5" => (
            "ollama-local",
            "codellama",
            "http://localhost:11434/v1",
            "chat_completions",
            false,
            None,
        ),
        "6" => {
            print!("\n  Enter provider name alias [custom-llm]: ");
            io::stdout().flush()?;
            let mut name_alias = String::new();
            reader.read_line(&mut name_alias)?;
            let name_alias = if name_alias.trim().is_empty() { "custom-llm".to_string() } else { name_alias.trim().to_string() };

            print!("  Enter model ID [gpt-4o]: ");
            io::stdout().flush()?;
            let mut model_id = String::new();
            reader.read_line(&mut model_id)?;
            let model_id = if model_id.trim().is_empty() { "gpt-4o".to_string() } else { model_id.trim().to_string() };

            print!("  Enter Base URL [http://localhost:8080/v1]: ");
            io::stdout().flush()?;
            let mut base_url = String::new();
            reader.read_line(&mut base_url)?;
            let base_url = if base_url.trim().is_empty() { "http://localhost:8080/v1".to_string() } else { base_url.trim().to_string() };

            return configure_provider(&name_alias, &model_id, &base_url, "chat_completions", true, None, &mut reader);
        }
        _ => {
            println!("Invalid choice.");
            return Ok(());
        }
    };

    configure_provider(key_name, default_model_id, base_url, default_api_backend, need_api_key, extra_header_key, &mut reader)
}

fn configure_provider(
    key_name: &str,
    model_id: &str,
    base_url: &str,
    api_backend: &str,
    need_api_key: bool,
    extra_header_key: Option<&str>,
    reader: &mut impl BufRead,
) -> Result<()> {
    let mut api_key = String::new();

    if need_api_key {
        print!("\n  Enter API Key for {key_name}: ");
        io::stdout().flush()?;
        reader.read_line(&mut api_key)?;
        api_key = api_key.trim().to_string();
        if api_key.is_empty() {
            println!("API key cannot be empty.");
            return Ok(());
        }
    }

    print!("\n  Set {key_name} as default model in config.toml? [Y/n]: ");
    io::stdout().flush()?;
    let mut set_def = String::new();
    reader.read_line(&mut set_def)?;
    let is_default = !set_def.trim().eq_ignore_ascii_case("n");

    save_to_config_toml(key_name, model_id, base_url, &api_key, api_backend, extra_header_key, is_default)?;

    println!("\n  ✓ Saved configuration for '{key_name}' to ~/.grok/config.toml!");
    if is_default {
        println!("  ✓ '{key_name}' is now set as the default model.");
    }
    println!("\n  You can run Anime now with:");
    println!("    anibuild -m {key_name}\n");

    Ok(())
}

fn save_to_config_toml(
    key_name: &str,
    model_id: &str,
    base_url: &str,
    api_key: &str,
    api_backend: &str,
    extra_header_key: Option<&str>,
    is_default: bool,
) -> Result<()> {
    let config_path = grok_home().join("config.toml");
    if let Some(parent) = config_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut doc = crate::config_toml_edit::read_config_document_for_edit(&config_path)
        .unwrap_or_else(toml_edit::DocumentMut::new);

    let mut model_table = toml_edit::Table::new();
    model_table["model"] = toml_edit::value(model_id);
    model_table["base_url"] = toml_edit::value(base_url);
    model_table["name"] = toml_edit::value(key_name);
    model_table["api_backend"] = toml_edit::value(api_backend);

    if !api_key.is_empty() {
        if let Some(hdr) = extra_header_key {
            let mut headers = toml_edit::InlineTable::new();
            headers.insert(hdr, toml_edit::Value::from(api_key));
            model_table["extra_headers"] = toml_edit::Item::Value(toml_edit::Value::InlineTable(headers));
        } else {
            model_table["api_key"] = toml_edit::value(api_key);
        }
    }

    doc["model"][key_name] = toml_edit::Item::Table(model_table);

    if is_default {
        doc["models"]["default"] = toml_edit::value(key_name);
    }

    std::fs::write(&config_path, doc.to_string())?;
    Ok(())
}
