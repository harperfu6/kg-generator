use serde::Deserialize;
use snafu::prelude::*;

use crate::{
    graph::{Graph, Triple},
    sparql::{self, sparql_req, Response, Value},
};

#[derive(Debug, Deserialize)]
struct SearchWord {
    word: String,
}

#[derive(Debug, Deserialize)]
struct Binding {
    p: Value,
    o: Value,
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

fn search_word_query(search_word: &str) -> String {
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

fn resp2triples(resp: Response<Binding>, search_word: &str) -> Vec<Triple> {
    let mut triples: Vec<Triple> = Vec::new();
    for binding in resp.results.bindings {
        triples.push(Triple {
            subject: search_word.to_string(),
            predicate: binding.p.value,
            object: binding.o.value,
        });
    }
    triples
}

pub async fn get_triples(search_word: &str) -> Result<Vec<Triple>, Error> {
    let swq = search_word_query(search_word);
    let resp = sparql_req::<Binding>(swq).await.context(SparqlSnafu)?;
    Ok(resp2triples(resp, search_word))
}

pub async fn get_graphs_from_file(file_path: &str) -> Result<Vec<Graph>, Error> {
    let search_words = read_search_word(file_path)?;

    let mut all_graph: Vec<Graph> = Vec::new();

    for search_word in search_words {
        let triples = get_triples(&search_word).await?;
        let graph = Graph::new(&search_word, triples);
        all_graph.push(graph);
    }

    Ok(all_graph)
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
        let file_path = "src/search_words.csv";
        let words = read_search_word(file_path).unwrap();
        assert_eq!(words.len(), 2);
        assert_eq!(words[0], "ローソン");
        assert_eq!(words[1], "ファミリーマート");
    }
}
