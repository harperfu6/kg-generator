use serde::Deserialize;
use snafu::prelude::*;

use crate::{
    graph::{Graph, Node, Triple},
    sparql::{self, sparql_req, Response, Value},
};

// sparql endpoint
const ENDPOINT: &str = "https://ja.dbpedia.org/sparql";

#[derive(Debug, Deserialize)]
struct SearchWord {
    word: String,
}

#[derive(Debug, Deserialize)]
struct Binding1Hop {
    p1: Value,
    o1: Value,
}

#[derive(Debug, Deserialize)]
struct Binding2Hop {
    p1: Value,
    o1: Value,
    p2: Value,
    o2: Value,
}

/// Read search words from csv file.
///
/// # Arguments
/// * `file_path` - A path of csv file.
pub fn read_search_word(file_path: &str) -> Result<Vec<String>, Error> {
    let mut words: Vec<String> = Vec::new();

    let mut rdr = csv::Reader::from_path(file_path).context(CsvSnafu)?;
    for result in rdr.deserialize() {
        let record: SearchWord = result.context(ReadCsvSnafu)?;
        words.push(record.word);
    }
    Ok(words)
}

fn query_1hop(
    search_word: &str,
    include_node_name_pattern: &Vec<&str>,
    exclude_node_name_pattern: &Vec<&str>,
) -> String {
    let include_node_name_pattern_str = include_node_name_pattern
        .iter()
        .map(|s| format!("(?={})", s))
        .collect::<Vec<String>>()
        .join("");
    let exclude_node_name_pattern_str = exclude_node_name_pattern
        .iter()
        .map(|s| format!("(?!{})", s))
        .collect::<Vec<String>>()
        .join("");

    let query = format!(
        r#"
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
        SELECT ?p1 ?o1
        WHERE {{
            <http://ja.dbpedia.org/resource/{}> ?p1 ?o1 .
            FILTER regex(?o1, "^{}{}")
        }}
        "#,
        search_word, include_node_name_pattern_str, exclude_node_name_pattern_str
    );
    query
}

fn query_2hop(search_word: &str) -> String {
    let query = format!(
        r#"
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
        SELECT ?p1 ?o1 ?p2 ?o2
        WHERE {{
            <http://ja.dbpedia.org/resource/{}> ?p1 ?o1 .
            ?o1 ?p2 ?o2 .
        }}
        "#,
        search_word
    );
    query
}

fn resp1hop2triples(resp: Response<Binding1Hop>, search_word: &str) -> Vec<Triple> {
    let mut triples: Vec<Triple> = Vec::new();
    for binding in resp.results.bindings {
        let p1 = binding.p1.value;
        let o1 = binding.o1.value;
        triples.push(Triple {
            subject: format!("http://ja.dbpedia.org/resource/{}", search_word.to_string()),
            predicate: p1,
            object: o1.to_string(),
        });
    }
    triples
}

fn resp2hop2triples(resp: Response<Binding2Hop>, search_word: &str) -> Vec<Triple> {
    let mut triples: Vec<Triple> = Vec::new();
    for binding in resp.results.bindings {
        let p1 = binding.p1.value;
        let o1 = binding.o1.value;
        let p2 = binding.p2.value;
        let o2 = binding.o2.value;
        triples.push(Triple {
            subject: format!("http://ja.dbpedia.org/resource/{}", search_word.to_string()),
            predicate: p1,
            object: o1.to_string(),
        });
        triples.push(Triple {
            subject: o1,
            predicate: p2,
            object: o2,
        });
    }
    triples
}

pub async fn get_triples(
    search_word: &str,
    include_node_name_pattern_list: &Vec<&str>,
    exclude_node_name_pattern_list: &Vec<&str>,
) -> Result<Vec<Triple>, Error> {
    let swq_1hop = query_1hop(
        search_word,
        include_node_name_pattern_list,
        exclude_node_name_pattern_list,
    );
    let resp1hop = sparql_req::<Binding1Hop>(ENDPOINT, swq_1hop)
        .await
        .context(SparqlSnafu)?;
    let hop1_triples = resp1hop2triples(resp1hop, search_word);

    Ok(hop1_triples)

    // let swq_2hop = query_2hop(search_word);
    // let resp2hop = sparql_req::<Binding2Hop>(ENDPOINT, swq_2hop)
    //     .await
    //     .context(SparqlSnafu)?;
    // let hop2_triples = resp2hop2triples(resp2hop, search_word);

    // Ok(hop1_triples
    //     .into_iter()
    //     .chain(hop2_triples.into_iter())
    //     .collect())
}

pub async fn get_graphs_from_search_words(
    search_words: Vec<String>,
    include_node_name_pattern_list: &Vec<&str>,
    exclude_node_name_pattern_list: &Vec<&str>,
) -> Result<Vec<Graph>, Error> {
    let mut all_graph: Vec<Graph> = Vec::new();

    for search_word in search_words {
        let triples = get_triples(
            &search_word,
            &include_node_name_pattern_list,
            &exclude_node_name_pattern_list,
        )
        .await?;
        let graph = Graph::new(&search_word, triples);

        all_graph.push(graph);
    }

    Ok(all_graph)
}

pub async fn get_graphs_from_file(
    file_path: &str,
    include_node_name_pattern_list: &Vec<&str>,
    remove_node_name_pattern_list: &Vec<&str>,
) -> Result<Vec<Graph>, Error> {
    let search_words = read_search_word(file_path)?;
    get_graphs_from_search_words(
        search_words,
        include_node_name_pattern_list,
        remove_node_name_pattern_list,
    )
    .await
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("csv error: {}", source))]
    Csv { source: csv::Error },
    #[snafu(display("csv error: {}", source))]
    ReadCsv { source: csv::Error },
    #[snafu(display("sparql error: {}", source))]
    Sparql { source: sparql::Error },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_csv() {
        let file_path = "input/search_words.csv";
        let words = read_search_word(file_path).unwrap();
        assert_eq!(words[0], "ローソン");
        assert_eq!(words[1], "ファミリーマート");
    }
}
