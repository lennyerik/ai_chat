use llm::LargeLanguageModel;

mod ddg;
mod llm;

fn main() -> Result<(), ddg::Error> {
    let mut chat = ddg::DDGChat::new(ddg::DDGChatModel::GPT4oMini)?;
    let mut resp = chat.send_message("hi!")?;

    for response_chunk in resp.flatten() {
        print!("{response_chunk}");
    }
    println!();

    Ok(())
}
