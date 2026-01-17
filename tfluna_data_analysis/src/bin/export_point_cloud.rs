use itertools::Itertools;
use std::error::Error;
use std::fs::File;

use polars::prelude::*;
use rerun::ChunkStoreConfig;
use rerun::dataframe::{QueryEngine, QueryExpression, SparseFillStrategy, TimelineName};
use rerun::external::arrow::compute::concat_batches;

use tfluna_data_analysis::convert::rerun_batches_to_polars;

fn main() -> Result<(), Box<dyn Error>> {
    let data_file = "data/pan_tilt.rrd";
    let timeline = TimelineName::log_time();
    let engines = QueryEngine::from_rrd_filepath(&ChunkStoreConfig::DEFAULT, data_file)?;

    for (store_id, engine) in engines {
        if store_id.is_empty_recording() {
            continue;
        }
        println!("Store id: {}", store_id);

        let query = QueryExpression {
            filtered_index: Some(timeline),
            sparse_fill_strategy: SparseFillStrategy::LatestAtGlobal,
            ..Default::default()
        };

        let query_handle = engine.query(query);
        let record_batches = query_handle.batch_iter().collect_vec();

        let batch = concat_batches(query_handle.schema(), &record_batches)?;
        println!("Converting data to polars dataframe");
        let batch_df = rerun_batches_to_polars(&vec![batch])?;
        // Extract the property columns (angle_step and servo_motor_delay) (they should be constant per recording)
        let batch_df: DataFrame = batch_df
            .lazy()
            .with_columns([
                col("property:angle_step:Scalars:scalars")
                    .list()
                    .first()
                    .alias("angle_step"),
                col("property:servo_motor_delay:Scalars:scalars")
                    .list()
                    .first()
                    .alias("servo_motor_delay"),
            ])
            .collect()?;

        let angle_step_val = batch_df
            .column("angle_step")?
            .f64()?
            .first()
            .ok_or(PolarsError::NoData("No data for angle_step".into()))?;
        let servo_delay_val = batch_df
            .column("servo_motor_delay")?
            .f64()?
            .first()
            .ok_or(PolarsError::NoData("No data for servo_motor_delay".into()))?;

        // Filtering and preparing dataframe
        let mut batch_df = batch_df.lazy().select([
            col("capture_time"),
            col("/distance:Scalars:scalars").list().first().alias("distance"),
            col("/pitch:Scalars:scalars").list().first().alias("pitch"),
            col("/signal_strength:Scalars:scalars").list().first().alias("signal_strength"),
            col("/temperature:Scalars:scalars").list().first().alias("temperature"),
            col("/yaw:Scalars:scalars").list().first().alias("yaw"),
            col("property:angle_step:Scalars:scalars").list().first().alias("angle_step"),
            col("property:servo_motor_delay:Scalars:scalars").list().first().alias("servo_motor_delay"),
            col("/position:Points3D:positions").list().last().arr().to_list().list().first().alias("x"),
            col("/position:Points3D:positions").list().last().arr().to_list().list().get(1.into(), true).alias("y"),
            col("/position:Points3D:positions").list().last().arr().to_list().list().last().alias("z"),
        ])
        .collect()?;
        println!("Dataframe: {}", batch_df);

        println!("Writing dataframe to csv file");
        let filepath = format!(
            "data/point_cloud_data_{}deg_{}ms.csv",
            angle_step_val, servo_delay_val
        );
        let mut file = File::create(filepath).expect("could not create file");
        CsvWriter::new(&mut file)
            .include_header(true)
            .with_separator(b',')
            .finish(&mut batch_df)?;

        //plot_point_cloud(point_cloud_positions, angle_step_val, servo_delay_val)?;
    }

    Ok(())
}
