mod graph;
mod read_words;
mod sparql;

use std::collections::HashMap;

use graph::{
    filter_by_target_nodes, get_target_nodes, group_by_node_count, remove_duplicate_node,
    remove_duplicates, save_as_n3, LinkAndNode,
};
use read_words::read_csv;
use regex::RegexSet;
use snafu::prelude::*;
use sparql::{parse_response, sparql_req};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let search_words = read_csv().context(ReadCsvSnafu)?;

    let mut all_link_nodes: HashMap<String, Vec<LinkAndNode>> = HashMap::new();
    let mut all_nodes: Vec<String> = Vec::new();

    for search_word in &search_words {
        let resp = sparql_req(&search_word).await.context(SparqlSnafu)?;

        let link_nodes = remove_duplicates(parse_response(resp));

        let nodes: Vec<String> = remove_duplicate_node(
            link_nodes
                .iter()
                .map(|link_node| link_node.node.clone())
                .collect(),
        );

        all_link_nodes.insert(search_word.clone(), link_nodes);
        all_nodes.extend(nodes);
    }

    let node_count = group_by_node_count(all_nodes);

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
    let target_nodes = get_target_nodes(
        node_count,
        node_count_thres,
        include_node_name_pattern,
        remove_node_name_pattern,
    );
    println!("{:#?}", target_nodes);

    let all_link_nodes_filtered = filter_by_target_nodes(all_link_nodes, target_nodes);
    println!("{:#?}", all_link_nodes_filtered);

    let file_name = "output.n3";
    save_as_n3(all_link_nodes_filtered, file_name).context(GraphWriteSnafu)?;

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
