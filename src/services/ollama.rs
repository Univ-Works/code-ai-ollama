use futures::stream::{self, Stream, StreamExt};
use ollama_rs::generation::chat::request::ChatMessageRequest;
use ollama_rs::generation::chat::{ChatMessage, MessageRole};
use ollama_rs::Ollama;
use tokio::io::AsyncWriteExt as _;
use tokio::sync::mpsc;
use std::pin::Pin;

use eval_code_ai::consts::{DEFAULT_SYSTEM_LOCK_CREATE_DYNAMIC_EXER, DEFAULT_SYSTEM_LOCK_CHAT, MODEL_CUSTOM};
use eval_code_ai::Result;

pub async fn chatting(
    message: String
) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
    let ollama = Ollama::default();

    let prompts = &[message];

    let system_msg = ChatMessage::new(MessageRole::System, DEFAULT_SYSTEM_LOCK_CHAT.to_string());
    let mut thread_msgs: Vec<ChatMessage> = vec![system_msg];

    for prompt in prompts {
        println!("\n->> prompt: {prompt}");

        let prompt_msg = ChatMessage::new(MessageRole::User, prompt.to_string());
        thread_msgs.push(prompt_msg);

        let chat_req = ChatMessageRequest::new(MODEL_CUSTOM.to_string(), thread_msgs.clone());
        let (tx, rx) = mpsc::channel(10);

        tokio::spawn(async move {
            let mut stream = ollama.send_chat_messages_stream(chat_req).await.unwrap();
            while let Some(res) = stream.next().await {
                let res = res.unwrap();
                if let Some(msg) = res.message {
                    let msg_content = msg.content;
                    if tx.send(Ok(msg_content)).await.is_err() {
                        break;
                    }
                }
                if let Some(_final_res) = res.final_data {
                    break;
                }
            }
        });

        return Ok(Box::pin(stream::unfold(rx, |mut rx| async {
            rx.recv().await.map(|msg| (msg, rx))
        })));
    }
    Ok(Box::pin(stream::empty()))
}

pub async fn create_dynamic_exercise(
    language: String,
    topic: String,
    sub_topic: String
) -> Result<()> {
    let ollama = Ollama::default();

    let formated = format!("language: {}, topic: {}, sub-topic: {}", language, topic, sub_topic);

    let prompts = &[formated];

    let system_msg = ChatMessage::new(MessageRole::System, DEFAULT_SYSTEM_LOCK_CREATE_DYNAMIC_EXER.to_string());
    let mut thread_msgs: Vec<ChatMessage> = vec![system_msg];

    for prompt in prompts {
        println!("\n->> prompt: {prompt}");

        let prompt_msg = ChatMessage::new(MessageRole::User, prompt.to_string());

        thread_msgs.push(prompt_msg);

        let chat_req = ChatMessageRequest::new(MODEL_CUSTOM.to_string(), thread_msgs.clone());

        let msg_content = run_chat_req(&ollama, chat_req).await?;

        if let Some(content) = msg_content {
            let asst_msg = ChatMessage::new(MessageRole::Assistant, content);
            thread_msgs.push(asst_msg);
        }
    }

    Ok(())
}

async fn run_chat_req(
	ollama: &Ollama,
	chat_req: ChatMessageRequest,
) -> Result<Option<String>> {
	let mut stream = ollama.send_chat_messages_stream(chat_req).await?;

	let mut stdout = tokio::io::stdout();
	let mut char_count = 0;
	let mut current_asst_msg_elems: Vec<String> = Vec::new();

	while let Some(res) = stream.next().await {
		let res = res.map_err(|_| "stream.next error")?;

		if let Some(msg) = res.message {
			let msg_content = msg.content;

			// Poor man's wrapping
			char_count += msg_content.len();
			if char_count > 80 {
				stdout.write_all(b"\n").await?;
				char_count = 0;
			}

			// Write output
			stdout.write_all(msg_content.as_bytes()).await?;
			stdout.flush().await?;

			current_asst_msg_elems.push(msg_content);
		}

		if let Some(_final_res) = res.final_data {
			stdout.write_all(b"\n").await?;
			stdout.flush().await?;

			let asst_content = current_asst_msg_elems.join("");
			return Ok(Some(asst_content));
		}
	}

	// new line
	stdout.write_all(b"\n").await?;
	stdout.flush().await?;

	Ok(None)
}

/*pub async fn run_ollama(request: String) -> Pin<Box<dyn Stream<Item = String> + Send>> {
    let ollama = Ollama::new("http://localhost".to_string(), 11434);

    let model = "seniordv:latest".to_string();
    let prompt = request.to_string();

    let (tx, rx) = mpsc::channel(20);

    tokio::spawn(async move {
        match ollama.generate(GenerationRequest::new(model, prompt)).await {
            Ok(res) => {
                for word in res.response.split_whitespace() {
                    if tx.send(word.to_string()).await.is_err() {
                        break;
                    }

                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                let _ = tx.send(format!("Error: {:?}", err)).await;
            }
        }
    });

    Box::pin(ReceiverStream::new(rx))
}*/