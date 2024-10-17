use std::{collections::HashMap, fs};

use cairo_lang_runner::profiling::ProfilingInfo;
use cairo_lang_sierra::program::Program;
use camino::Utf8PathBuf;

/// Saves post-processed stack trace with weights
pub fn save_profiler_output(program: &Program, profiling_info: &ProfilingInfo, output_path: &Utf8PathBuf) -> anyhow::Result<()> {
    let format_stack_item = |(idx_stack_trace, weight): (&Vec<usize>, &usize)| {
        let key = index_stack_trace_to_name_stack_trace(program, &idx_stack_trace).join(";");
        format!("{key} {weight}")
    };

    let contents = profiling_info
        .stack_trace_weights
        .iter()
        .map(format_stack_item)
        .collect::<Vec::<String>>()
        .join("\n");

    fs::write(output_path, contents)?;
    Ok(())
}

fn process(program: &Program, profiling_info: &ProfilingInfo) -> Vec<(Vec<String>, usize)> {

    for (stack_trace, weight) in profiling_info.stack_trace_weights.iter() {
        let mut rec_depth: HashMap<String, usize> = Default::default();
        let mut key_parts: Vec<String> = Vec::with_capacity(stack_trace.len());
        for func_idx in stack_trace {
            let func_name = program.funcs[*func_idx].id.to_string();
            if let Some(depth) = rec_depth.get_mut(&func_name) {
                *depth += 1;
            } else {
                key_parts.push(func_name.clone());
                rec_depth.insert(func_name, 1);
            }
        }
    }
    vec![]
}

/// Converts a stack trace represented as a vector of indices of functions in the sierra program to
/// a stack trace represented as a vector of function names.
/// Assumes that the given `idx_stack_trace` is valid with respect to the given `sierra_program`.
/// That is, each index in the stack trace is within range of the sierra program.
fn index_stack_trace_to_name_stack_trace(
    sierra_program: &Program,
    idx_stack_trace: &[usize],
) -> Vec<String> {
    idx_stack_trace.iter().map(|idx| sierra_program.funcs[*idx].id.to_string()).collect()
}
