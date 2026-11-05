use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use cesium_math::{TaskError, TaskProcessor, WasmOptions};
use serde_json::{json, Value};

#[test]
fn works_with_a_simple_worker() {
    let mut processor = TaskProcessor::new("returnParameters.js", None);

    processor.set_handler(Arc::new(|params, _| {
        // Echoes parameters back
        Ok((params, None))
    }));

    let parameters = json!({
        "prop": "blah",
        "obj": {
            "val": true
        }
    });

    let result = processor
        .schedule_task(parameters.clone(), None)
        .unwrap()
        .unwrap();

    assert_eq!(result.0, parameters);
    assert!(result.1.is_none());
}

#[test]
fn respects_maximum_active_tasks_limit() {
    let mut processor = TaskProcessor::new("worker.js", Some(0));

    processor.set_handler(Arc::new(|params, _| Ok((params, None))));

    // Since maximum_active_tasks is 0, schedule_task returns None (throttled)
    let opt = processor.schedule_task(json!({}), None).unwrap();
    assert!(opt.is_none());
}

#[test]
fn can_transfer_array_buffer_and_return_back() {
    let mut processor = TaskProcessor::new("transferBuffer.js", None);

    processor.set_handler(Arc::new(|_, maybe_buf| {
        let mut buf = maybe_buf.unwrap();
        // Modify bytes in place
        for b in &mut buf {
            *b = b.wrapping_add(10);
        }
        Ok((json!({"byte_len": buf.len()}), Some(buf)))
    }));

    let input_bytes = vec![1, 2, 3, 4, 5];
    let (meta, returned_buf) = processor
        .schedule_task(json!({}), Some(input_bytes))
        .unwrap()
        .unwrap();

    assert_eq!(meta["byte_len"], 5);
    assert_eq!(returned_buf, Some(vec![11, 12, 13, 14, 15]));
}

#[test]
fn rejects_if_worker_throws() {
    let mut processor = TaskProcessor::<Value, Value>::new("throwingWorker.js", None);

    processor.set_handler(Arc::new(|_, _| {
        Err(TaskError::runtime_error("Worker computation failed"))
    }));

    let res = processor.schedule_task(json!({}), None);
    assert!(res.is_err());
    assert!(res.unwrap_err().message.contains("Worker computation failed"));
}

#[test]
fn successful_task_raises_task_completed_event() {
    let mut processor = TaskProcessor::new("worker.js", None);
    let event_fired = Arc::new(AtomicBool::new(false));
    let ef = event_fired.clone();

    processor.add_task_completed_listener(Arc::new(move |err| {
        assert!(err.is_none());
        ef.store(true, Ordering::SeqCst);
    }));

    processor.set_handler(Arc::new(|params, _| Ok((params, None))));

    processor.schedule_task(json!({"ok": true}), None).unwrap();
    assert!(event_fired.load(Ordering::SeqCst));
}

#[test]
fn unsuccessful_task_raises_task_completed_event_with_error() {
    let mut processor = TaskProcessor::<Value, Value>::new("worker.js", None);
    let error_caught = Arc::new(AtomicBool::new(false));
    let ec = error_caught.clone();

    processor.add_task_completed_listener(Arc::new(move |err| {
        if let Some(e) = err {
            assert_eq!(e.message, "Boom");
            ec.store(true, Ordering::SeqCst);
        }
    }));

    processor.set_handler(Arc::new(|_, _| {
        Err(TaskError::runtime_error("Boom"))
    }));

    assert!(processor.schedule_task(json!({}), None).is_err());
    assert!(error_caught.load(Ordering::SeqCst));
}

#[test]
fn wasm_loader_config_with_wasm_supported() {
    let processor = TaskProcessor::<(), ()>::new("wasmWorker.js", None);
    let options = WasmOptions {
        wasm_binary_file: Some("build/module.wasm".to_string()),
        fallback_module_path: Some("build/fallback.js".to_string()),
    };

    let config = processor.get_wasm_loader_config(true, &options).unwrap();
    assert_eq!(config.wasm_binary_file, Some("build/module.wasm".to_string()));
    assert!(config.module_path.is_none());
}

#[test]
fn wasm_loader_config_uses_backup_if_wasm_unsupported() {
    let processor = TaskProcessor::<(), ()>::new("wasmWorker.js", None);
    let options = WasmOptions {
        wasm_binary_file: Some("build/module.wasm".to_string()),
        fallback_module_path: Some("build/fallback.js".to_string()),
    };

    let config = processor.get_wasm_loader_config(false, &options).unwrap();
    assert_eq!(config.module_path, Some("build/fallback.js".to_string()));
    assert!(config.wasm_binary_file.is_none());
}

#[test]
fn wasm_loader_config_throws_if_wasm_unsupported_and_no_backup() {
    let processor = TaskProcessor::<(), ()>::new("wasmWorker.js", None);
    let options = WasmOptions {
        wasm_binary_file: Some("build/module.wasm".to_string()),
        fallback_module_path: None,
    };

    let res = processor.get_wasm_loader_config(false, &options);
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .message
        .contains("does not support Web Assembly, and no backup module was provided"));
}

#[test]
fn task_processor_can_be_destroyed() {
    let mut processor = TaskProcessor::<Value, Value>::new("worker.js", None);
    assert!(!processor.is_destroyed());

    processor.destroy();
    assert!(processor.is_destroyed());

    let res = processor.schedule_task(json!({}), None);
    assert!(res.is_err());
}
