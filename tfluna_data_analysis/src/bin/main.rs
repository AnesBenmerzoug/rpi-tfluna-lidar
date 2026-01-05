use itertools::Itertools;
use std::error::Error;
use std::io::Cursor;

use polars::prelude::*;
use rerun::ChunkStoreConfig;
use rerun::dataframe::{QueryEngine, QueryExpression, SparseFillStrategy, TimelineName};
use rerun::external::arrow::ipc::writer::FileWriter;
use rerun::external::arrow::{array::RecordBatch, compute::concat_batches};

use tfluna_pan_tilt::evaluate::{
    analyze_experiment, calculate_repeatability, plot_error_scatter,
};

fn main() -> Result<(), Box<dyn Error>> {
    let data_file = "data/pan_tilt_combinations.rrd.rrd";
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
    let results_df = analyze_experiment(data_df)?;
    println!("Calculating repeatability metrics");
    let repeatability_df = calculate_repeatability(&results_df)?;
    println!("Results: {}", results_df);
    println!("Repeatability: {}", repeatability_df);
    println!("Plotting results");
    plot_error_scatter(&results_df)?;

    Ok(())
}

fn rerun_batches_to_polars(batches: &[RecordBatch]) -> PolarsResult<DataFrame> {
    if batches.is_empty() {
        return Err(PolarsError::ComputeError("No batches provided".into()));
    }

    // Serialize all batches with Rerun's Arrow
    let mut buffer = Vec::new();
    {
        let mut writer = FileWriter::try_new(&mut buffer, &batches[0].schema())
            .map_err(|e| PolarsError::ComputeError(e.to_string().into()))?;

        for batch in batches {
            writer
                .write(batch)
                .map_err(|e| PolarsError::ComputeError(e.to_string().into()))?;
        }
        writer
            .finish()
            .map_err(|e| PolarsError::ComputeError(e.to_string().into()))?;
    }

    // Deserialize directly into Polars DataFrame
    let cursor = Cursor::new(buffer);
    let df = IpcReader::new(cursor).finish()?;

    Ok(df)
}
