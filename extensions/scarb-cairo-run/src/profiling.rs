use std::fs;

use cairo_lang_runner::profiling::ProfilingInfo;
use cairo_lang_sierra::program::{GenStatement, Program};
use camino::Utf8PathBuf;
use itertools::Itertools;

/// Saves aggregated stack trace with weights
pub fn save_profiler_output(
    program: &Program,
    profiling_info: &ProfilingInfo,
    output_path: &Utf8PathBuf,
) -> anyhow::Result<()> {
    let contents = profiling_info
        .scoped_sierra_statement_weights
        .iter()
        .map(|((stack_trace, statement_idx), weight)| {
            let key = stack_trace
                .iter()
                .map(|idx| program.funcs[*idx].id.to_string())
                .join(";");

            let statement = program
                .statements
                .get(statement_idx.0)
                .expect("non-existing statement");

            let stmt = match statement {
                GenStatement::Invocation(inv) => inv
                    .libfunc_id
                    .debug_name
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or("call"),
                GenStatement::Return(_) => "ret",
            };

            format!("{key};{stmt}_[k] {weight}")
        })
        .join("\n");

    fs::write(output_path, contents)?;
    Ok(())
}
