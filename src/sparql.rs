use serde::Deserialize;
use snafu::prelude::*;

#[derive(Debug, Deserialize)]
pub struct Value {
    #[serde(rename = "type")]
    pub typ: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
struct Header {
    link: Vec<String>,
    vars: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Results<B> {
    pub distinct: bool,
    pub ordered: bool,
    pub bindings: Vec<B>,
}

#[derive(Debug, Deserialize)]
pub struct Response<B> {
    head: Header,
    pub results: Results<B>,
}

/// Send a SPARQL query to the endpoint.
///
/// # Arguments
/// * `endpoint` - A SPARQL endpoint.
/// * `query` - A SPARQL query.
pub async fn sparql_req<B>(endpoint: &str, query: String) -> Result<Response<B>, Error>
where
    B: for<'de> Deserialize<'de>,
{
    let params = [
        ("query", query),
        ("format", "json".to_string()),
        ("timeout", "30000".to_string()),
    ];

    let resp = reqwest::Client::new()
        .get(endpoint)
        .query(&params)
        .send()
        .await
        .context(ReqwestSnafu)?
        .json::<Response<B>>()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_sparql_req() {
        #[derive(Debug, Deserialize)]
        struct Binding {
            p: Value,
            o: Value,
        }
        let endpoint = "http://ja.dbpedia.org/sparql";

        let query = r#"
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
            SELECT ?p ?o
            WHERE {
                <http://ja.dbpedia.org/resource/日本> ?p ?o .
            }
        "#;
        let resp = sparql_req::<Binding>(endpoint, query.to_string())
            .await
            .unwrap();
        println!("{:?}", resp);
    }
}
