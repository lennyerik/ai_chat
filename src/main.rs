use llm::LargeLanguageModel;

mod ddg;
mod llm;

fn main() -> Result<(), ddg::Error> {
    let mut chat = ddg::DDGChat::new(ddg::DDGChatModel::GPT4oMini)?;
    chat.send_message("hi!")?;

    for response_chunk in chat.flatten() {
        print!("{response_chunk}");
    }
    println!();

    Ok(())
}
