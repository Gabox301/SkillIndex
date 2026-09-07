use skillindex::ui::{show_cursor, write};

pub fn handle_sigint() {
    tokio::spawn(async {
        let _ = tokio::signal::ctrl_c().await;
        write(&format!("{}\n", show_cursor()));
        std::process::exit(130);
    });
}
