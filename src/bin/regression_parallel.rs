//! Parallel regression test runner with process isolation and timeouts.
//!
//! This runner executes all regression tests in parallel, spawning worker processes
//! for each test with OS-level timeouts and memory limits.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use wait_timeout::ChildExt;

#[derive(Debug, Clone)]
struct TestCase {
    name: String,
    path: PathBuf,
}

#[derive(Debug)]
#[allow(dead_code)]
enum TestStatus {
    Success,
    Timeout,
    Failed(String),
}

#[derive(Debug)]
struct TestResult {
    name: String,
    status: TestStatus,
    duration: Duration,
}

fn discover_tests() -> Vec<TestCase> {
    let examples_dir = PathBuf::from("_wb/foras-examples");
    let mut tests = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&examples_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                // Scan subdirectories for .in files
                if let Ok(files) = std::fs::read_dir(&path) {
                    for file in files.filter_map(|f| f.ok()) {
                        let file_path = file.path();
                        if file_path.extension().and_then(|s| s.to_str()) == Some("in") {
                            let name = file_path.file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string();
                            tests.push(TestCase {
                                name,
                                path: file_path,
                            });
                        }
                    }
                }
            }
        }
    }

    tests.sort_by(|a, b| a.name.cmp(&b.name));
    tests
}

fn run_test(test: &TestCase, timeout_secs: u64, memory_limit_mb: usize) -> TestResult {
    let start = Instant::now();

    // Spawn worker process
    let worker_bin = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|parent| parent.join("regression_worker")))
        .expect("Failed to locate regression_worker binary");

    let mut child = Command::new(&worker_bin)
        .arg(test.path.to_str().unwrap())
        .arg(timeout_secs.to_string())
        .arg(memory_limit_mb.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn worker");

    // Wait with timeout
    let timeout_duration = Duration::from_secs(timeout_secs + 2); // +2s grace period
    match child.wait_timeout(timeout_duration).unwrap() {
        Some(status) => {
            let duration = start.elapsed();
            let status_result = if status.success() {
                TestStatus::Success
            } else {
                let code = status.code().unwrap_or(-1);
                TestStatus::Failed(format!("Exit code: {}", code))
            };

            TestResult {
                name: test.name.clone(),
                status: status_result,
                duration,
            }
        }
        None => {
            // Timeout - kill the process
            let _ = child.kill();
            let _ = child.wait();

            TestResult {
                name: test.name.clone(),
                status: TestStatus::Timeout,
                duration: timeout_duration,
            }
        }
    }
}

fn main() {
    println!("=== Parallel Regression Test Runner ===\n");

    let timeout_secs = 10;      // Increased from 5s - equational tests need more time
    let memory_limit_mb = 4096; // Increased from 1GB - Knuth-Bendix needs ~3-4GB
    let parallelism = num_cpus::get();

    println!("Configuration:");
    println!("  Timeout: {}s per test", timeout_secs);
    println!("  Memory limit: {}MB per test", memory_limit_mb);
    println!("  Parallelism: {} cores\n", parallelism);

    // Discover tests
    println!("Discovering tests...");
    let tests = discover_tests();
    println!("Found {} test cases\n", tests.len());

    if tests.is_empty() {
        eprintln!("No test cases found in _wb/foras-examples/");
        std::process::exit(1);
    }

    // Run tests in parallel
    println!("Running tests...\n");
    let start = Instant::now();

    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];

    // Use a simple thread pool approach
    let tests_arc = Arc::new(tests);
    let test_index = Arc::new(Mutex::new(0));

    for _ in 0..parallelism {
        let tests_clone = Arc::clone(&tests_arc);
        let results_clone = Arc::clone(&results);
        let index_clone = Arc::clone(&test_index);

        let handle = std::thread::spawn(move || {
            loop {
                let idx = {
                    let mut index = index_clone.lock().unwrap();
                    let current = *index;
                    *index += 1;
                    current
                };

                if idx >= tests_clone.len() {
                    break;
                }

                let test = &tests_clone[idx];
                let result = run_test(test, timeout_secs, memory_limit_mb);

                // Print progress
                println!("[{}/{}] {} - {:?} ({:.2}s)",
                         idx + 1,
                         tests_clone.len(),
                         result.name,
                         result.status,
                         result.duration.as_secs_f64());

                results_clone.lock().unwrap().push(result);
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    let total_duration = start.elapsed();
    let results = Arc::try_unwrap(results).unwrap().into_inner().unwrap();

    // Print summary
    println!("\n=== RESULTS ===");
    let total = results.len();
    let success = results.iter().filter(|r| matches!(r.status, TestStatus::Success)).count();
    let timeouts = results.iter().filter(|r| matches!(r.status, TestStatus::Timeout)).count();
    let failures = results.iter().filter(|r| matches!(r.status, TestStatus::Failed(_))).count();

    println!("Total tests: {}", total);
    println!("Success: {} ({:.1}%)", success, (success as f64 / total as f64) * 100.0);
    println!("Timeouts: {}", timeouts);
    println!("Failures: {}", failures);
    println!("\nTotal time: {:.2}s", total_duration.as_secs_f64());
    println!("Average per test: {:.2}s", total_duration.as_secs_f64() / total as f64);

    let sequential_time = total as f64 * timeout_secs as f64;
    println!("Speedup vs sequential: {:.1}x", sequential_time / total_duration.as_secs_f64());

    // Exit with error if there were failures
    if timeouts > 0 || failures > 0 {
        println!("\nFailed tests:");
        for result in &results {
            if !matches!(result.status, TestStatus::Success) {
                println!("  - {} ({:?})", result.name, result.status);
            }
        }
        std::process::exit(1);
    }

    println!("\nAll tests passed!");
}
