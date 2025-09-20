use std::io;
use std::env;
use crate::ai::{ask_question, create_client, load_devlog_context, read_ai_config};
use crate::utils::devlog_path;

pub fn run_ai_mode() -> io::Result<()> {
    // Read config
    let devlog_path = devlog_path();
    let cfg = read_ai_config(&devlog_path)?;
    let api_key = env::var("OPENAI_API_KEY")
        .ok()
        .or(cfg.openai_api_key)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "OpenAI API key not set. Set OPENAI_API_KEY env or write openai_api_key in .devlog/config.toml",
            )
        })?;
    let model = cfg.model.unwrap_or_else(|| "gpt-4o-mini".to_string());

    // Initialize client and load context
    let client = create_client(&api_key);
    let context = load_devlog_context(&devlog_path, 200_000).unwrap_or_default();

    println!("devlog ai — ask about files in .devlog (type 'exit' to quit)\n");
    let system_prefix = "You are a helpful assistant that answers questions about the user's devlog notes. Base your answers strictly on the provided files. If unsure, say you don't know.";
    let full_context = format!("{}\n\nHere are the devlog files:\n{}", system_prefix, context);

    // Simple REPL loop
    use std::io::Write;
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    
    // Create a single runtime for the entire session
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Tokio runtime error: {}", e)))?;

    loop {
        print!(">> ");
        let _ = stdout.flush();
        let mut q = String::new();
        if stdin.read_line(&mut q)? == 0 {
            break;
        }
        let q = q.trim();
        if q.is_empty() {
            continue;
        }
        if q.eq_ignore_ascii_case("exit") || q.eq_ignore_ascii_case("quit") {
            break;
        }

        match rt.block_on(ask_question(&client, &model, &full_context, q)) {
            Ok(response) => println!("\n{}\n", response.trim()),
            Err(e) => println!("\nError: {}\n", e),
        }
    }

    Ok(())
}
