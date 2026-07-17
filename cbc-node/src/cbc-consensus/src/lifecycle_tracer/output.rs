// Trace output destinations

use crate::lifecycle_tracer::error::OutputError;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Trait for trace output destinations
pub trait TraceOutput: Send + Sync {
    /// Write a formatted entry to output destination
    fn write(&self, formatted_entry: &str) -> Result<(), OutputError>;
    /// Flush output destination to ensure traces are persisted
    fn flush(&self) -> Result<(), OutputError>;
}

/// Stdout output implementation
pub struct StdoutOutput;

impl TraceOutput for StdoutOutput {
    fn write(&self, formatted_entry: &str) -> Result<(), OutputError> {
        println!("{}", formatted_entry);
        Ok(())
    }

    fn flush(&self) -> Result<(), OutputError> {
        use std::io::Write;
        std::io::stdout()
            .flush()
            .map_err(OutputError::StdoutWriteError)
    }
}

/// File output implementation with buffered writing
pub struct FileOutput {
    #[allow(dead_code)]
    path: PathBuf,
    writer: Arc<Mutex<BufWriter<File>>>,
}

impl FileOutput {
    /// Create a new file output destination
    pub fn new(path: PathBuf) -> Result<Self, OutputError> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| OutputError::FileWriteError(e))?;
        }

        // Open file in append mode with create flag
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| OutputError::FileWriteError(e))?;

        // Set file permissions to 0600 (owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = file.metadata()
                .map_err(|e| OutputError::FileWriteError(e))?
                .permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&path, perms)
                .map_err(|e| OutputError::FileWriteError(e))?;
        }

        let writer = BufWriter::new(file);

        Ok(Self {
            path,
            writer: Arc::new(Mutex::new(writer)),
        })
    }
}

impl TraceOutput for FileOutput {
    fn write(&self, formatted_entry: &str) -> Result<(), OutputError> {
        let mut writer = self.writer.lock().unwrap();
        writeln!(writer, "{}", formatted_entry)
            .map_err(|e| OutputError::FileWriteError(e))?;
        Ok(())
    }

    fn flush(&self) -> Result<(), OutputError> {
        let mut writer = self.writer.lock().unwrap();
        writer.flush()
            .map_err(|e| OutputError::FileWriteError(e))
    }
}

use once_cell::sync::Lazy;
use prometheus::Opts;

static LIFECYCLE_STEP_COUNTER: Lazy<prometheus::Counter> = Lazy::new(|| {
    prometheus::register_counter!(
        Opts::new("cbc_lifecycle_step_total", "Total lifecycle trace steps processed")
    ).unwrap()
});

static LIFECYCLE_LAST_STEP: Lazy<prometheus::Gauge> = Lazy::new(|| {
    prometheus::register_gauge!(
        Opts::new("cbc_lifecycle_last_step", "Last step number reached in node lifecycle")
    ).unwrap()
});

static LIFECYCLE_ERRORS: Lazy<prometheus::Counter> = Lazy::new(|| {
    prometheus::register_counter!(
        Opts::new("cbc_lifecycle_errors_total", "Total error traces processed by lifecycle tracer")
    ).unwrap()
});

static LIFECYCLE_MILESTONES: Lazy<prometheus::Counter> = Lazy::new(|| {
    prometheus::register_counter!(
        Opts::new("cbc_lifecycle_milestones_total", "Total milestone traces processed")
    ).unwrap()
});

/// Metrics output implementation (placeholder for Prometheus integration)
pub struct MetricsOutput;

impl TraceOutput for MetricsOutput {
    fn write(&self, formatted_entry: &str) -> Result<(), OutputError> {
        // 1. Try to parse the trace entry as JSON first
        if let Ok(entry) = serde_json::from_str::<serde_json::Value>(formatted_entry) {
            LIFECYCLE_STEP_COUNTER.inc();
            if let Some(entry_type) = entry.get("type").and_then(|v| v.as_str()) {
                match entry_type {
                    "step" => {
                        if let Some(step) = entry.get("step").and_then(|v| v.as_u64()) {
                            LIFECYCLE_LAST_STEP.set(step as f64);
                        }
                    }
                    "milestone" => {
                        LIFECYCLE_MILESTONES.inc();
                    }
                    "error" => {
                        LIFECYCLE_ERRORS.inc();
                    }
                    _ => {}
                }
            }
        } else {
            // Fallback: simple text parsing for human-readable format
            LIFECYCLE_STEP_COUNTER.inc();
            if formatted_entry.contains("🎯 MILESTONE") {
                LIFECYCLE_MILESTONES.inc();
            } else if formatted_entry.contains("❌ ERROR") {
                LIFECYCLE_ERRORS.inc();
                // Extract step number: e.g. "ERROR at STEP 5:"
                if let Some(idx) = formatted_entry.find("ERROR at STEP ") {
                    let start = idx + "ERROR at STEP ".len();
                    if let Some(end) = formatted_entry[start..].find(':') {
                        if let Ok(step) = formatted_entry[start..start+end].trim().parse::<f64>() {
                            LIFECYCLE_LAST_STEP.set(step);
                        }
                    }
                }
            } else if formatted_entry.contains("[CBC-TRACE]") {
                // Extract step number: e.g. "[CBC-TRACE] 5."
                if let Some(idx) = formatted_entry.find("[CBC-TRACE] ") {
                    let start = idx + "[CBC-TRACE] ".len();
                    if let Some(end) = formatted_entry[start..].find('.') {
                        if let Ok(step) = formatted_entry[start..start+end].trim().parse::<f64>() {
                            LIFECYCLE_LAST_STEP.set(step);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn flush(&self) -> Result<(), OutputError> {
        // No-op for metrics output
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_stdout_output() {
        let output = StdoutOutput;
        assert!(output.write("Test message").is_ok());
        assert!(output.flush().is_ok());
    }

    #[test]
    fn test_file_output_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        
        let output = FileOutput::new(path.clone());
        assert!(output.is_ok());
    }

    #[test]
    fn test_file_output_write() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        
        let output = FileOutput::new(path.clone()).unwrap();
        assert!(output.write("Test line 1").is_ok());
        assert!(output.write("Test line 2").is_ok());
        assert!(output.flush().is_ok());
        
        // Read the file and verify contents
        let mut contents = String::new();
        let mut file = File::open(&path).unwrap();
        file.read_to_string(&mut contents).unwrap();
        
        assert!(contents.contains("Test line 1"));
        assert!(contents.contains("Test line 2"));
    }

    #[test]
    fn test_file_output_creates_directories() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("subdir").join("trace.log");
        
        let output = FileOutput::new(path.clone());
        assert!(output.is_ok());
        assert!(path.exists());
    }

    #[test]
    fn test_metrics_output() {
        let output = MetricsOutput;
        assert!(output.write("Test metric").is_ok());
        assert!(output.flush().is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn test_file_permissions() {
        use std::os::unix::fs::PermissionsExt;
        
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        
        let _output = FileOutput::new(path.clone()).unwrap();
        
        let metadata = std::fs::metadata(&path).unwrap();
        let permissions = metadata.permissions();
        let mode = permissions.mode();
        
        // Check that only owner has read/write permissions (0600)
        assert_eq!(mode & 0o777, 0o600);
    }
}
