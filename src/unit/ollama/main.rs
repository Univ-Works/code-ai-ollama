use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};

#[tokio::main]
async fn main() {
    let ollama = Ollama::new("http://localhost".to_string(), 11434);

    let model = "llama3:latest".to_string();
    let prompt = "Cuenta del 1 al 10".to_string();

    let res = ollama.generate(GenerationRequest::new(model, prompt)).await;

    /*if let Ok(res) = res {
        println!("Response: {}", res.response);
        if let Some(duration_ns) = res. {
            let duration_sec = duration_ns as f64 / 1_000_000_000.0;
            println!("Duration in seconds: {:.6}", duration_sec);
        }
    } else if let Err(err) = res {
        eprintln!("Error: {:?}", err)
    }*/
}