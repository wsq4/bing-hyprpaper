mod args;
mod download;
mod model;
mod app;
mod hyprpaper;

use clap::Parser;
use daemonize::Daemonize;
use crate::args::Args;

#[tokio::main(flavor = "multi_thread")]
async fn main_loop(args : &Args) {
    let app = app::App::new(args).expect("Failed to initialize application");

    app.run().await.expect("Failed to run application");
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    if args.daemon {
        println!("Running in daemon mode...");
        
        let daemonize = Daemonize::new()
            .pid_file("/tmp/bing_hyprpaper.pid")
            .chown_pid_file(true)
            .working_directory("/tmp")
            .umask(0o027)
            .stdout(std::fs::File::create("/tmp/bing_hyprpaper.out").unwrap())
            .stderr(std::fs::File::create("/tmp/bing_hyprpaper.err").unwrap());

        match daemonize.start() {
            Ok(_) => {
                println!("Daemon started successfully");
                main_loop(&args);
            },
            Err(e) => panic!("Error, {}", e),
        }

    } else {
        println!("Running in one-time mode...");
        main_loop(&args);
    }
}