use serde::Deserialize;
use snafu::prelude::*;

use crate::graph::LinkAndNode;

// sparql endpoint
const ENDPOINT: &str = "https://ja.dbpedia.org/sparql";

#[derive(Debug, Deserialize)]
struct Value {
    #[serde(rename = "type")]
    typ: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct Binding {
    p: Value,
    o: Value,
}

#[derive(Debug, Deserialize)]
struct Header {
    link: Vec<String>,
    vars: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Results {
    distinct: bool,
    ordered: bool,
    bindings: Vec<Binding>,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    head: Header,
    results: Results,
}

fn generate_query(search_word: &str) -> String {
    let query = format!(
        r#"
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
        SELECT ?p ?o
        WHERE {{
            <http://ja.dbpedia.org/resource/{}> ?p ?o .
        }}
        "#,
        search_word
    );
    query
}

pub async fn sparql_req(search_word: &str) -> Result<Response, Error> {
    let params = [
        ("query", generate_query(search_word)),
        ("format", "json".to_string()),
        ("timeout", "30000".to_string()),
    ];

    let resp = reqwest::Client::new()
        .get(ENDPOINT)
        .query(&params)
        .send()
        .await
        .context(ReqwestSnafu)?
        .json::<Response>()
        .await
        .context(JsonSnafu)?;

    Ok(resp)
}

pub fn parse_response(resp: Response) -> Vec<LinkAndNode> {
    let mut link_nodes: Vec<LinkAndNode> = Vec::new();
    for binding in resp.results.bindings {
        link_nodes.push(LinkAndNode {
            link: binding.p.value,
            node: binding.o.value,
        });
    }

    link_nodes
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("reqwest error: {}", source))]
    Reqwest { source: reqwest::Error },
    #[snafu(display("json error: {}", source))]
    Json { source: reqwest::Error },
}
