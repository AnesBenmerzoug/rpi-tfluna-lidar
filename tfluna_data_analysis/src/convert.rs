use std::io::Cursor;

use polars::prelude::*;
use rerun::external::arrow::array::RecordBatch;
use rerun::external::arrow::ipc::writer::FileWriter;

pub fn rerun_batches_to_polars(batches: &[RecordBatch]) -> PolarsResult<DataFrame> {
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
