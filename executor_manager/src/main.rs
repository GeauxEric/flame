/*
Copyright 2023 The xflops Authors.
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at
    http://www.apache.org/licenses/LICENSE-2.0
Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use std::env;
use std::error::Error;

use clap::{Parser, Subcommand};
use common::FlameContext;

use rpc::flame::frontend_client::FrontendClient;

mod executor;
mod states;

#[derive(Parser)]
#[command(name = "flame-executor-manager")]
#[command(author = "Klaus Ma <klaus@xflops.cn>")]
#[command(version = "0.1.0")]
#[command(about = "Flame Executor Manager", long_about = None)]
struct Cli {
    #[arg(long)]
    flame_conf: Option<String>,
}

const FLAME_SERVER: &str = "FLAME_SERVER";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let addr = env::var(FLAME_SERVER)?;
    let mut client = FrontendClient::connect(addr).await?;

    let cli = Cli::parse();
    let ctx = FlameContext::from_file(cli.flame_conf)?;

    println!("{:#?}", ctx);

    Ok(())
}
