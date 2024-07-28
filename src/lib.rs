mod error;

pub use self::error::{Error, Result};

pub mod splitter;

pub mod consts {
    pub const MODEL_CUSTOM: &str = "gemma";

    pub const DEFAULT_SYSTEM_LOCK_CHAT: &str = r#"
        You only know Data Structures and Object-Oriented Programming topics, and you are a senior developer.
        Also you know three languages, such as English, Spanish and Russian.
    "#;

    pub const DEFAULT_SYSTEM_LOCK_CREATE_DYNAMIC_EXER: &str = r#"
        You only have to send 1 problem, could be code or theoretical concepts about Objected-Oriented Programming and Data Structrures topics.
        The problem should be a sentence and 3 options, only one of them options is correct.
        Make sure of show the correct option with this parameter: "ans: ".
        Remember, I'll give you a topic for you create the problem with its options.  

        Here the sub-topics of each one (OOP and Data Structures):
        - OOP: Encapsulation, abstraction, inheritance and polimorfism.
        - Data Structures: Arrays, single linked list, double linked list, circular linked list,
        stacks, queues, trees and graphs.

        This is an strict pattern that I'll send you. Thi is an example:
        language: spanish || english || russian, topic: OOP || DS, sub-topic: sub-topics.

        Note: You are a senior developer and many programming languages and 
        you know english, spanish and russian.
    "#;
}

pub mod gen {
	use super::*;
	use futures::StreamExt;
	use ollama_rs::generation::completion::request::GenerationRequest;
	use ollama_rs::generation::completion::GenerationFinalResponseData;
	use ollama_rs::Ollama;
	use tokio::io::AsyncWriteExt;

	/// NOTE: OLLAMA-RS 0.1.6 now returns a Vec<GenerationResponse>, so handling accordingly.
	/// TODO: Need to further understand what does the Vec<GenerationResponse> v.s. the old GenerationResponse
	///       means to refine behavior. See ticket: https://github.com/pepperoni21/ollama-rs/issues/20)
	pub async fn gen_stream_print(
		ollama: &Ollama,
		gen_req: GenerationRequest,
	) -> Result<Vec<GenerationFinalResponseData>> {
		let mut stream = ollama.generate_stream(gen_req).await?;

		let mut stdout = tokio::io::stdout();
		let mut char_count = 0;

		let mut final_data_responses = Vec::new();

		while let Some(res) = stream.next().await {
			// NOTE: For now, we just flatten this result list since it will most likely be a vec of one.
			//       However, if res.length > 1, we might want to split the output, as those might be for different responses.
			let res_list = res?;

			for res in res_list {
				let bytes = res.response.as_bytes();

				// Poor man's wrapping
				char_count += bytes.len();
				if char_count > 80 {
					stdout.write_all(b"\n").await?;
					char_count = 0;
				}

				// Write output
				stdout.write_all(bytes).await?;
				stdout.flush().await?;

				if let Some(data) = res.final_data {
					//stdout.write_all(data.response.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
					stdout.flush().await?;
					final_data_responses.push(data);
					break;
				}
			}
		}

		stdout.write_all(b"\n").await?;
		stdout.flush().await?;

		Ok(final_data_responses)
	}
}