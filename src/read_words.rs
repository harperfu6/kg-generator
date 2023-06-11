use serde::Deserialize;
use snafu::prelude::*;

#[derive(Debug, Deserialize)]
struct SearchWord {
    word: String,
}

pub fn read_csv() -> Result<Vec<String>, Error> {
    let mut words: Vec<String> = Vec::new();

    let mut rdr = csv::Reader::from_path("src/search_words.csv").context(CsvSnafu)?;
    for result in rdr.deserialize() {
        let record: SearchWord = result.context(ReadCsvSnafu)?;
        words.push(record.word);
    }
    Ok(words)
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("csv error: {}", source))]
    Csv { source: csv::Error },
    #[snafu(display("csv error: {}", source))]
    ReadCsv { source: csv::Error },
}

#[test]
fn test_read_csv() {
    let words = read_csv().unwrap();
    assert_eq!(words.len(), 2);
    assert_eq!(words[0], "ローソン");
    assert_eq!(words[1], "ファミリーマート");
}
