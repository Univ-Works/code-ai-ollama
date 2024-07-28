use actix::{Actor, Addr, AsyncContext, Context, Message, Handler, StreamHandler};
use eval_code_ai::splitter::simple_char_splitter;
use futures::StreamExt;
use actix_web_actors::ws;

use super::ollama::{chatting, create_dynamic_exercise};
pub struct MyWs {
    addr: Option<Addr<MyWs>>
}

impl MyWs {
    pub fn new() -> Self {
        Self { addr: None }
    }
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.addr = Some(ctx.address());
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct OllamaResponse(String);

impl Handler<OllamaResponse> for MyWs {
    type Result = ();

    fn handle(&mut self, msg: OllamaResponse, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                let text_clone_for_async = text.clone();
                if let Some(addr) = self.addr.clone() {
                    actix::spawn(async move {
                        match chatting(text_clone_for_async.to_string()).await {
                            Ok(mut response_stream) => {
                                while let Some(Ok(partial_text)) = response_stream.next().await {
                                    if let Ok(splitter) = simple_char_splitter(&partial_text, 1) {
                                        for chunk in splitter {
                                            addr.do_send(OllamaResponse(chunk));
                                        }
                                    }
                                }
                            },
                            Err(e) => {
                                addr.do_send(OllamaResponse(format!("Error: {}", e)));
                            }
                        }
                    });
                }
            },
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}