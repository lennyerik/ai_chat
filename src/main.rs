use fallible_iterator::FallibleIterator;
use std::io::Write;

use llm::LargeLanguageModel;

mod ddg;
mod llm;

fn run_model_and_print_response<'m, M: llm::LargeLanguageModel<'m>>(
    model: &'m mut M,
    prompt: &str,
) -> Result<(), <M as LargeLanguageModel<'m>>::Error> {
    let mut response = model.send_message(prompt)?;

    let mut stdout = std::io::stdout().lock();
    while let Some(response_chunk) = response.next()? {
        let _ = write!(stdout, "{response_chunk}");
        let _ = stdout.flush();
    }
    println!();

    Ok(())
}

fn exit_with_error_msg(err_msg: &str) -> ! {
    eprintln!("{err_msg}");
    std::process::exit(1)
}

macro_rules! exit_with_error {
    () => {
        |e| exit_with_error_msg(&format!("{e}"))
    };
}

fn main() {
    let prompt: String = std::env::args().skip(1).collect();
    let prompt = prompt.trim();

    if prompt.is_empty() {
        exit_with_error_msg(&format!(
            "No message supplied.\nUsage: {} <MESSAGE>",
            std::env::args().next().unwrap_or_else(|| "./ai".to_owned())
        ));
    }

    let mut model =
        ddg::DDGChat::new(ddg::DDGChatModel::GPT4oMini).unwrap_or_else(exit_with_error!());
    run_model_and_print_response(&mut model, prompt).unwrap_or_else(exit_with_error!());
}
