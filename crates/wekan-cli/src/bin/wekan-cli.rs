use wekan_cli::{display::CliDisplay, runner::Runner};

#[tokio::main]
async fn main() {
    let mut c = Runner::new().await;
    std::process::exit(<Runner as CliDisplay>::transform_to_exit(c.run().await).into());
}
