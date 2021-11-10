use clap::{AppSettings, Clap};
use anyhow::Result;
use anyhow::anyhow;
use colored::*;
use reqwest::{Url, Client, Response, Version, StatusCode};
use std::convert::TryInto;
use std::time::Duration;
use std::collections::HashMap;
use reqwest::header::HeaderMap;

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

    try_print_response(response).await;
    Ok(())
}

async fn try_print_response(response: Response) -> Result<()> {
    let version = response.version();
    let status_code = response.status();
    let header_map = response.headers().clone();
    let text = response.text().await?;
    let response_bundle = ResponseBundle(version, status_code, header_map, text);
    print_response(response_bundle)
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
    try_print_response(response).await?;
    Ok(())
}

struct ResponseBundle(Version, StatusCode, HeaderMap, String);

fn print_response(response_bundle: ResponseBundle) -> Result<()> {
    // print status line
    let status_line = format!("{:?} {}\n", response_bundle.0, response_bundle.1).blue();
    println!("{}", status_line);

    // print header line
    let mut pretty_style = Option::None;
    for header in response_bundle.2 {
        match header.0 {
            Some(ref header_name) => {
                let header_line = format!("{:?} : {:?}", header_name, header.1).green();
                println!("{}", header_line);
                if header_name == "application/json" {
                    pretty_style = Some(true);
                }
            }
            _ => {}
        }
    }

    match pretty_style {
        Some(_) => {
            println!("\n{}", response_bundle.3);
        }

        None => {
            println!("\n{}", jsonxf::pretty_print(response_bundle.3.as_str()).unwrap().cyan());
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    println!("{:?}", opts);

    let client = Client::new();
    match opts.subcmd {
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