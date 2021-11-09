use clap::{AppSettings, Clap};
use anyhow::Result;
use anyhow::anyhow;
use reqwest::{Url, Client, Response};
use std::convert::TryInto;
use std::time::Duration;
use std::collections::HashMap;

#[derive(Debug, Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Debug, Clap)]
enum SubCommand {
    Get(Get),
    Post(Post),
}

#[derive(Debug, Clap)]
struct Get {
    #[clap(parse(try_from_str = parse_url))]
    url: String,
}

#[derive(Debug, Clap)]
struct Post {
    url: String,
    #[clap(parse(try_from_str = parse_body))]
    body: Vec<BodyFormPair>,
}

#[derive(Debug)]
struct BodyFormPair {
    k: String,
    v: String,
}


fn parse_url(s: &str) -> Result<String> {
    let _url = s.parse::<Url>()?;
    Ok(s.to_string())
}

fn parse_body(s: &str) -> Result<BodyFormPair> {
    Ok(s.try_into()?)
}

impl TryInto<BodyFormPair> for &str {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<BodyFormPair, Self::Error> {
        let mut split = self.split("=");
        let err = || anyhow!(format!("Failed to parse {}", self));
        Ok(BodyFormPair {
            k: (split.next().ok_or_else(err)?).to_string(),
            v: (split.next().ok_or_else(err)?).to_string(),
        })
    }
}

async fn get(client: &Client, get: &Get) -> Result<()> {
    let response = client
        .get(&get.url)
        .timeout(Duration::from_secs(10))
        .send().await?;
    println!("{}", response.text().await?);
    Ok(())
}

async fn post(client: &Client, post: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for pair in post.body.iter() {
        body.insert(&pair.k, &pair.v);
    }
    let response = client
        .post(&post.url)
        .json(&body)
        .timeout(Duration::from_secs(10))
        .send().await?;
    println!("{}", response.text().await?);
    Ok(())
}



#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    println!("{:?}", opts);

    let client = Client::new();
    let response = match opts.subcmd {
        SubCommand::Get(data) => get(&client, &data).await?,
        SubCommand::Post(data) => post(&client, &data).await?,
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_opts_parse() {
        use super::*;
        let opts: Opts = Opts::parse();
        println!("{:?}", opts);
    }
}