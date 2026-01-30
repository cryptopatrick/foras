/// Minimal test to isolate hanging issue
use foras::regression::ExampleSuite;
use std::io::Write;

fn main() {
    let mut stderr = std::io::stderr();

    writeln!(stderr, "Step 1: Starting...").ok();
    stderr.flush().ok();

    writeln!(stderr, "Step 2: Creating suite...").ok();
    stderr.flush().ok();
    let suite = ExampleSuite::default();

    writeln!(stderr, "Step 3: Suite created").ok();
    stderr.flush().ok();

    writeln!(stderr, "Step 4: Getting available inputs...").ok();
    stderr.flush().ok();
    match suite.available_inputs() {
        Ok(inputs) => {
            writeln!(stderr, "Step 5: Found {} inputs", inputs.len()).ok();
            stderr.flush().ok();
        }
        Err(e) => {
            writeln!(stderr, "Step 5: ERROR: {}", e).ok();
            stderr.flush().ok();
            return;
        }
    }

    writeln!(stderr, "Step 6: Success!").ok();
    stderr.flush().ok();
}
