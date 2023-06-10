mod read_words;
mod sparql;

use snafu::prelude::*;
use sparql::sparql_req;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let resp = sparql_req().await.context(SparqlSnafu)?;
    println!("{:#?}", resp);
    Ok(())
}

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("sparql error: {}", source))]
    Sparql { source: sparql::Error },
}
