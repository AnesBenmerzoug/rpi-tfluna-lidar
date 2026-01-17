use itertools::Itertools;
use std::error::Error;
use std::fs::File;

use polars::prelude::*;
use rerun::ChunkStoreConfig;
use rerun::dataframe::{QueryEngine, QueryExpression, SparseFillStrategy, TimelineName};
use rerun::external::arrow::compute::concat_batches;

use tfluna_data_analysis::convert::rerun_batches_to_polars;
use tfluna_data_analysis::evaluate::{analyze_experiment, calculate_repeatability};
use tfluna_data_analysis::plot::plot_error_scatter;

fn main() -> Result<(), Box<dyn Error>> {
    let data_file = "data/pan_tilt_combinations.rrd";
    let timeline = TimelineName::log_time();
    let engines = QueryEngine::from_rrd_filepath(&ChunkStoreConfig::DEFAULT, data_file)?;

    println!("Querying and concatenating data");
    // Collected data
    let mut all_batches = Vec::new();

    for (store_id, engine) in engines {
        if store_id.is_empty_recording() {
            continue;
        }

        let query = QueryExpression {
            filtered_index: Some(timeline),
            sparse_fill_strategy: SparseFillStrategy::LatestAtGlobal,
            ..Default::default()
        };

        let query_handle = engine.query(query);
        let record_batches = query_handle.batch_iter().collect_vec();

        let batch = concat_batches(query_handle.schema(), &record_batches)?;
        all_batches.push(batch);
    }

    println!("Converting data to polars dataframe");
    let data_df = rerun_batches_to_polars(&all_batches)?;
    println!("dataframe: {}", data_df);

    println!("Analyzing data");
    let mut metrics_df = analyze_experiment(data_df)?;
    println!("Calculating repeatability metrics");
    let mut repeatability_df = calculate_repeatability(&metrics_df)?;
    println!("Metrics: {}", metrics_df);
    println!("Repeatability: {}", repeatability_df);
    println!("Saving results");
    let mut metrics_file = File::create("data/metrics.csv").expect("could not create file");
    CsvWriter::new(&mut metrics_file)
        .include_header(true)
        .with_separator(b',')
        .finish(&mut metrics_df)?;
    
    let mut repeatability_file = File::create("data/repeatability.csv").expect("could not create file");
    CsvWriter::new(&mut repeatability_file)
        .include_header(true)
        .with_separator(b',')
        .finish(&mut repeatability_df)?;
    //plot_error_scatter(&results_df)?;

    Ok(())
}
