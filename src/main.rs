mod graph;
mod read_words;
mod sparql;

use graph::{get_target_nodes, reduce_node_counts, Graph, NodeCount};
use read_words::read_csv;
use regex::RegexSet;
use snafu::prelude::*;
use sparql::{parse_response, sparql_req};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let search_words = read_csv().context(ReadCsvSnafu)?;

    let mut all_graph: Vec<Graph> = Vec::new();
    let mut node_counts_vec: Vec<Vec<NodeCount>> = Vec::new();

    for search_word in &search_words {
        let resp = sparql_req(&search_word).await.context(SparqlSnafu)?;

        let graph = Graph::new(&search_word, parse_response(resp));
        let node_counts = graph.group_by_node_count();

        all_graph.push(graph);
        node_counts_vec.push(node_counts);
    }

    let node_count_thres = &search_words.len();
    let include_node_name_pattern =
        RegexSet::new(&[r"http://ja.dbpedia.org/resource/+"]).context(RegexSnafu)?;
    let remove_node_name_pattern = RegexSet::new(&[
        r"http://ja.dbpedia.org/resource/(\d{1})月(\d{1})日",
        r"http://ja.dbpedia.org/resource/(\d{1})月(\d{2})日",
        r"http://ja.dbpedia.org/resource/(\d{2})月(\d{1})日",
        r"http://ja.dbpedia.org/resource/(\d{2})月(\d{2})日",
        r"http://ja.dbpedia.org/resource/(\d{4})年",
        r"http://ja.dbpedia.org/resource/Template:+",
    ])
    .context(RegexSnafu)?;

    let all_node_counts = reduce_node_counts(node_counts_vec);
    let target_nodes = get_target_nodes(
        all_node_counts,
        node_count_thres,
        include_node_name_pattern,
        remove_node_name_pattern,
    );
    println!("{:#?}", target_nodes);

    for graph in all_graph {
        println!("graph_name: {}", graph.graph_name);
        let graph_filtered = graph.filter_by_target_nodes(&target_nodes);

        let file_name = format!("{}/{}.n3", "data", graph.graph_name);
        graph_filtered
            .save_as_n3(&file_name)
            .context(GraphWriteSnafu)?;
    }

    Ok(())
}

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("sparql error: {}", source))]
    Sparql { source: sparql::Error },
    #[snafu(display("csv error: {}", source))]
    ReadCsv { source: read_words::Error },
    #[snafu(display("regex error: {}", source))]
    Regex { source: regex::Error },
    #[snafu(display("io error: {}", source))]
    GraphWrite { source: graph::GraphWriteError },
}
