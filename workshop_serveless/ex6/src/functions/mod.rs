// WARNING: This file is regenerated by the `cargo func new` command.
use azure_functions::codegen::Function;

mod queue;
mod queue_with_output;

// Export the Azure Functions here.
pub const EXPORTS: &[&Function] = &[
    &queue::QUEUE_FUNCTION,
    &queue_with_output::QUEUE_WITH_OUTPUT_FUNCTION,
];
