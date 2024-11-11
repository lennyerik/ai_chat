use std::{
    io::{BufRead, BufReader, Read},
    ops::Not,
};

use super::llm;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("network request error")]
    Ureq(#[from] Box<ureq::Error>),

    #[error("no new Vqd received with last request")]
    NoVqdReceived,

    #[error("got end of string while reading web response")]
    ResponseEndOfString,

    #[error("received invalid response")]
    ResponseInvalid,
}

#[derive(serde::Serialize, Debug, Clone, Copy)]
pub enum DDGChatModel {
    #[serde(rename = "gpt-4o-mini")]
    GPT4oMini,

    #[serde(rename = "claude-3-haiku-20240307")]
    Claude3Haiku,

    #[serde(rename = "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo")]
    Llama370B,

    #[serde(rename = "mistralai/Mixtral-8x7B-Instruct-v0.1")]
    Mixtral8x7B,
}

#[derive(serde::Serialize, Debug, Clone, Copy)]
pub enum MessageRole {
    #[serde(rename = "user")]
    User,

    #[serde(rename = "asssistant")]
    Assistant,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct DDGMessage {
    role: MessageRole,
    content: String,
}

#[derive(Debug)]
pub struct DDGChat {
    model: DDGChatModel,
    messages: Vec<DDGMessage>,
    current_vqd: String,
}

impl DDGChat {
    pub fn new(model: DDGChatModel) -> Result<Self, Error> {
        let vqd_resp = ureq::get("https://duckduckgo.com/duckchat/v1/status")
            .set("User-Agent", USER_AGENT)
            .set("X-Vqd-Accept", "1")
            .call()
            .map_err(Box::new)?;
        let vqd = vqd_resp.header("X-Vqd-4").ok_or(Error::NoVqdReceived)?;

        Ok(Self {
            model,
            messages: Vec::new(),
            current_vqd: vqd.to_owned(),
        })
    }
}

impl<'c> llm::LargeLanguageModel<'c> for DDGChat {
    type Error = Error;
    type Response = DDGResponse<'c>;

    fn send_message(&'c mut self, message: &str) -> Result<Self::Response, Self::Error> {
        self.messages.push(DDGMessage {
            role: MessageRole::User,
            content: message.to_string(),
        });

        let net_resp = ureq::post("https://duckduckgo.com/duckchat/v1/chat")
            .set("User-Agent", USER_AGENT)
            .set("X-Vqd-4", &self.current_vqd)
            .send_json(ureq::json!({
                "model": self.model,
                "messages": self.messages,
            }))
            .map_err(Box::new)?;

        self.current_vqd = net_resp
            .header("X-Vqd-4")
            .unwrap_or(&self.current_vqd)
            .to_string();

        Ok(DDGResponse {
            chat: self,
            content: String::new(),
            reader: BufReader::new(net_resp.into_reader()),
            result: Ok(()),
        })
    }
}

pub struct DDGResponse<'c> {
    chat: &'c mut DDGChat,
    content: String,
    reader: BufReader<Box<dyn std::io::Read + Send + Sync + 'static>>,
    result: Result<(), Error>,
}

impl DDGResponse<'_> {
    fn finish(&mut self) {
        self.chat.messages.push(DDGMessage {
            role: MessageRole::Assistant,
            content: std::mem::take(&mut self.content),
        });
    }
}

fn get_message_from_data_str(data: &str) -> Option<String> {
    let data = data.strip_prefix("data: ")?;
    let json: serde_json::Value = serde_json::from_str(data).ok()?;
    let message = json.get("message")?.as_str()?;
    Some(message.to_owned())
}

impl Iterator for DDGResponse<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.result.is_err() {
            return None;
        }

        let line = {
            let mut buf = String::new();
            self.reader.read_line(&mut buf).ok()?;
            buf = buf.trim_end().to_string();

            // Skip the following newline
            let _ = self.reader.read(&mut [0u8; 1]);

            buf.is_empty().not().then_some(buf)
        };

        if let Some(line) = line {
            if line == "[DONE]" {
                self.finish();
                return None;
            }

            let message = get_message_from_data_str(&line);

            if let Some(message) = message {
                self.content.push_str(&message);
                Some(message)
            } else {
                self.result = Err(Error::ResponseInvalid);
                self.finish();
                None
            }
        } else {
            self.result = Err(Error::ResponseEndOfString);
            self.finish();
            None
        }
    }
}

impl llm::LLMResponse<Error> for DDGResponse<'_> {
    fn result(self) -> Result<(), Error> {
        self.result
    }
}
