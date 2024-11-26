use std::{
    io::{BufRead, BufReader, Read},
    ops::Not,
};

use serde::{Deserialize, Serialize};

use super::llm;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("network request error: {0}")]
    Ureq(#[from] Box<ureq::Error>),

    #[error("no new Vqd received with last request")]
    NoVqdReceived,

    #[error("got end of string while reading web response")]
    ResponseEndOfString,

    #[error("received invalid response '{0}'")]
    ResponseInvalid(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum DDGChatModel {
    #[default]
    #[serde(rename = "gpt-4o-mini")]
    GPT4oMini,

    #[serde(rename = "claude-3-haiku-20240307")]
    Claude3Haiku,

    #[serde(rename = "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo")]
    Llama370B,

    #[serde(rename = "mistralai/Mixtral-8x7B-Instruct-v0.1")]
    Mixtral8x7B,

    #[serde(untagged)]
    Other(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum MessageRole {
    #[serde(rename = "user")]
    User,

    #[serde(rename = "asssistant")]
    Assistant,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

impl<'r> llm::LargeLanguageModel<'r> for DDGChat {
    type Error = Error;
    type Response = DDGResponse<'r>;

    fn send_message(&'r mut self, message: &str) -> Result<Self::Response, Self::Error> {
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
        })
    }
}

pub struct DDGResponse<'c> {
    chat: &'c mut DDGChat,
    content: String,
    reader: BufReader<Box<dyn std::io::Read + Send + Sync + 'static>>,
}

impl DDGResponse<'_> {
    fn get_next_line(&mut self) -> Option<String> {
        let mut buf = String::new();
        self.reader.read_line(&mut buf).ok()?;
        buf = buf.trim_end().to_string();

        // Skip the following newline
        let _ = self.reader.read(&mut [0u8; 1]);

        buf.is_empty().not().then_some(buf)
    }

    fn finish(&mut self) {
        self.chat.messages.push(DDGMessage {
            role: MessageRole::Assistant,
            content: std::mem::take(&mut self.content),
        });
    }
}

fn parse_message_data(data: &str) -> Result<Option<String>, ()> {
    let json: serde_json::Value = serde_json::from_str(data).map_err(|_| ())?;

    let action = json.get("action").ok_or(())?.as_str().ok_or(())?;
    if action != "success" {
        return Err(());
    }

    match json.get("message") {
        Some(message) => Ok(Some(message.as_str().ok_or(())?.to_owned())),
        None => Ok(None),
    }
}

impl fallible_iterator::FallibleIterator for DDGResponse<'_> {
    type Item = String;
    type Error = Error;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        let line = self.get_next_line().ok_or(Error::ResponseEndOfString)?;
        let data = line.strip_prefix("data: ").unwrap_or(&line);

        if data == "[DONE]" {
            self.finish();
            return Ok(None);
        }

        let message = parse_message_data(data).map_err(|()| Error::ResponseInvalid(line))?;

        if let Some(message) = &message {
            self.content.push_str(message);
        } else {
            self.finish();
        }
        Ok(message)
    }
}
