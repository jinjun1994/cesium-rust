use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::RuntimeError;

/// Worker module prefix configuration.
pub static DEFAULT_WORKER_MODULE_PREFIX: &str = "Workers/";

/// Represents an error raised during task execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskError {
    pub name: String,
    pub message: String,
    pub stack: Option<String>,
}

impl TaskError {
    pub fn runtime_error(msg: impl Into<String>) -> Self {
        Self {
            name: "RuntimeError".to_string(),
            message: msg.into(),
            stack: None,
        }
    }
}

/// Options for configuring WebAssembly loader in TaskProcessor.
#[derive(Debug, Clone, Default)]
pub struct WasmOptions {
    pub wasm_binary_file: Option<String>,
    pub fallback_module_path: Option<String>,
}

/// Resulting WebAssembly loader configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmLoaderConfig {
    pub module_path: Option<String>,
    pub wasm_binary_file: Option<String>,
    pub wasm_binary: Option<Vec<u8>>,
}

/// Event listener callback when a task completes (successfully or with error).
pub type TaskCompletedCallback = Arc<dyn Fn(Option<&TaskError>) + Send + Sync>;

/// Result data and optional transferred-back buffer from a task.
pub type TaskResult<R> = (R, Option<Vec<u8>>);

/// Scheduled task output result.
pub type TaskScheduleResult<R> = Result<Option<TaskResult<R>>, RuntimeError>;

/// Synchronous or simulated worker execution handler.
pub type WorkerHandler<P, R> = Arc<dyn Fn(P, Option<Vec<u8>>) -> Result<TaskResult<R>, TaskError> + Send + Sync>;

/// A wrapper around worker tasks that schedules tasks and returns results.
pub struct TaskProcessor<P = serde_json::Value, R = serde_json::Value> {
    worker_path: String,
    maximum_active_tasks: usize,
    active_tasks: usize,
    next_id: usize,
    destroyed: bool,
    handler: Option<WorkerHandler<P, R>>,
    task_completed_listeners: Vec<TaskCompletedCallback>,
}

impl<P, R> TaskProcessor<P, R> {
    /// Creates a new TaskProcessor with the specified worker path and maximum active tasks.
    pub fn new(worker_path: impl Into<String>, maximum_active_tasks: Option<usize>) -> Self {
        Self {
            worker_path: worker_path.into(),
            maximum_active_tasks: maximum_active_tasks.unwrap_or(usize::MAX),
            active_tasks: 0,
            next_id: 0,
            destroyed: false,
            handler: None,
            task_completed_listeners: Vec::new(),
        }
    }

    /// Sets the worker execution handler (used for test simulation or custom backend workers).
    pub fn set_handler(&mut self, handler: WorkerHandler<P, R>) {
        self.handler = Some(handler);
    }

    /// Adds a listener to be notified when a task completes.
    pub fn add_task_completed_listener(&mut self, listener: TaskCompletedCallback) {
        self.task_completed_listeners.push(listener);
    }

    /// The worker path.
    pub fn worker_path(&self) -> &str {
        &self.worker_path
    }

    /// Current number of active tasks.
    pub fn active_tasks(&self) -> usize {
        self.active_tasks
    }

    /// Maximum number of active tasks.
    pub fn maximum_active_tasks(&self) -> usize {
        self.maximum_active_tasks
    }

    /// Returns true if the processor has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }

    /// Destroys the task processor.
    pub fn destroy(&mut self) {
        self.destroyed = true;
        self.task_completed_listeners.clear();
        self.handler = None;
    }

    /// Schedules a task. Returns None if active tasks exceed maximum.
    pub fn schedule_task(
        &mut self,
        parameters: P,
        transferable_buffer: Option<Vec<u8>>,
    ) -> TaskScheduleResult<R> {
        if self.destroyed {
            return Err(RuntimeError::new(
                "Cannot schedule task on destroyed TaskProcessor",
            ));
        }

        if self.active_tasks >= self.maximum_active_tasks {
            return Ok(None);
        }

        self.active_tasks += 1;
        self.next_id += 1;

        let result = if let Some(ref handler) = self.handler {
            handler(parameters, transferable_buffer)
        } else {
            Err(TaskError::runtime_error("No worker handler installed"))
        };

        self.active_tasks -= 1;

        match result {
            Ok((res, transferred_back)) => {
                for listener in &self.task_completed_listeners {
                    listener(None);
                }
                Ok(Some((res, transferred_back)))
            }
            Err(err) => {
                for listener in &self.task_completed_listeners {
                    listener(Some(&err));
                }
                Err(RuntimeError::new(err.message))
            }
        }
    }

    /// Resolves WebAssembly configuration given support and options.
    pub fn get_wasm_loader_config(
        &self,
        supports_wasm: bool,
        options: &WasmOptions,
    ) -> Result<WasmLoaderConfig, RuntimeError> {
        if !supports_wasm {
            if let Some(ref fallback) = options.fallback_module_path {
                return Ok(WasmLoaderConfig {
                    module_path: Some(fallback.clone()),
                    wasm_binary_file: None,
                    wasm_binary: None,
                });
            } else {
                return Err(RuntimeError::new(format!(
                    "This browser does not support Web Assembly, and no backup module was provided for {}",
                    self.worker_path
                )));
            }
        }

        Ok(WasmLoaderConfig {
            module_path: None,
            wasm_binary_file: options.wasm_binary_file.clone(),
            wasm_binary: None,
        })
    }
}
