mod graph;
mod sparql;
mod word_search;

use graph::{get_target_nodes, reduce_node_counts, Graph, NodeCount};
use regex::RegexSet;
use snafu::prelude::*;

use word_search::get_graphs_from_file;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let all_graph = get_graphs_from_file("data/search_words.csv")
        .await
        .context(WordSearchSnafu)?;

    // // ここから以下は知識グラフとして結合することを目的とした処理
    // let node_count_thres = &search_words.len();
    // let include_node_name_pattern =
    //     RegexSet::new(&[r"http://ja.dbpedia.org/resource/+"]).context(RegexSnafu)?;
    // let remove_node_name_pattern = RegexSet::new(&[
    //     r"http://ja.dbpedia.org/resource/(\d{1})月(\d{1})日",
    //     r"http://ja.dbpedia.org/resource/(\d{1})月(\d{2})日",
    //     r"http://ja.dbpedia.org/resource/(\d{2})月(\d{1})日",
    //     r"http://ja.dbpedia.org/resource/(\d{2})月(\d{2})日",
    //     r"http://ja.dbpedia.org/resource/(\d{4})年",
    //     r"http://ja.dbpedia.org/resource/Template:+",
    // ])
    // .context(RegexSnafu)?;

    // let all_node_counts = reduce_node_counts(node_counts_vec);
    // let target_nodes = get_target_nodes(
    //     all_node_counts,
    //     node_count_thres,
    //     include_node_name_pattern,
    //     remove_node_name_pattern,
    // );
    // println!("{:#?}", target_nodes);

    for graph in all_graph {
        println!("graph_name: {}", graph.graph_name);
        // let graph_filtered = graph.filter_by_target_nodes(&target_nodes);

        let file_name = format!("{}/{}/{}.n3", "output", "rdf", graph.graph_name);
        // graph_filtered
        graph.save_as_n3(&file_name).context(GraphWriteSnafu)?;
    }

    Ok(())
}

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("word searh error: {}", source))]
    WordSearch { source: word_search::Error },
    #[snafu(display("sparql error: {}", source))]
    Sparql { source: sparql::Error },
    #[snafu(display("regex error: {}", source))]
    Regex { source: regex::Error },
    #[snafu(display("io error: {}", source))]
    GraphWrite { source: graph::GraphWriteError },
}
