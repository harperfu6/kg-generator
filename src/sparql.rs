use serde::Deserialize;
use snafu::prelude::*;

// sparql endpoint
const ENDPOINT: &str = "https://ja.dbpedia.org/sparql";

// sparql select example
const QUERY: &str = "select * where { <http://ja.dbpedia.org/resource/宮崎駿> ?p ?o . } limit 5";
// curl -X GET https://ja.dbpedia.org/sparql\?query\="select%20%2A%20where%20%7B%20%3Chttp%3A%2F%2Fja.dbpedia.org%2Fresource%2F%E5%AE%AE%E5%B4%8E%E9%A7%BF%3E%20%3Fp%20%3Fo%20.%20%7D%20limit%205"\&format\=json

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

pub async fn sparql_req() -> Result<Response, Error> {
    let params = [("query", QUERY), ("format", "json"), ("timeout", "30000")];

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

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("reqwest error: {}", source))]
    Reqwest { source: reqwest::Error },
    #[snafu(display("json error: {}", source))]
    Json { source: reqwest::Error },
}
