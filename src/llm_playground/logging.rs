// Enhanced logging system for process observability and performance monitoring
use serde_json::json;
use wasm_bindgen::{JsValue, JsCast};

/// Logging levels for different types of events
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Performance,
    UserAction,
    ApiCall,
    FunctionCall,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Performance => "PERF",
            LogLevel::UserAction => "USER",
            LogLevel::ApiCall => "API",
            LogLevel::FunctionCall => "FUNC",
        }
    }

    fn color(&self) -> &'static str {
        match self {
            LogLevel::Debug => "color: #888",
            LogLevel::Info => "color: #0066cc",
            LogLevel::Warn => "color: #ff9900",
            LogLevel::Error => "color: #cc0000",
            LogLevel::Performance => "color: #9900cc",
            LogLevel::UserAction => "color: #00cc66",
            LogLevel::ApiCall => "color: #cc6600",
            LogLevel::FunctionCall => "color: #6600cc",
        }
    }
}

/// Enhanced logger with structured logging
#[derive(Clone)]
pub struct ProcessLogger {
    session_id: Option<String>,
    component: String,
    start_time: f64,
}

impl ProcessLogger {
    pub fn new(component: &str) -> Self {
        Self {
            session_id: None,
            component: component.to_string(),
            start_time: js_sys::Date::now(),
        }
    }

    pub fn with_session(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }

    /// Log structured data with context
    pub fn log(&self, level: LogLevel, event: &str, data: Option<serde_json::Value>) {
        let timestamp = js_sys::Date::now();
        let duration = timestamp - self.start_time;
        
        let log_entry = json!({
            "timestamp": timestamp,
            "duration_ms": duration,
            "level": level.as_str(),
            "component": self.component,
            "session_id": self.session_id,
            "event": event,
            "data": data
        });

        // Enhanced console logging with styling
        let prefix = format!("[{}] [{}]", level.as_str(), self.component);
        let session_info = self.session_id
            .as_ref()
            .map(|id| format!(" [Session: {}]", &id[..8]))
            .unwrap_or_default();
        
        let message = if let Some(data) = data {
            format!("{}{} {}: {}", prefix, session_info, event, 
                serde_json::to_string_pretty(&data).unwrap_or_default())
        } else {
            format!("{}{} {}", prefix, session_info, event)
        };

        // Use styled console output
        web_sys::console::log_2(
            &JsValue::from_str(&format!("%c{}", message)),
            &JsValue::from_str(level.color()),
        );

        // Also store in browser's performance timeline for analysis (if available)
        // Note: Performance API might not be available in all environments
    }

    /// Log user interactions
    pub fn log_user_action(&self, action: &str, details: Option<serde_json::Value>) {
        self.log(LogLevel::UserAction, action, details);
    }

    /// Log API calls with timing
    pub fn log_api_call(&self, endpoint: &str, method: &str, details: Option<serde_json::Value>) {
        let data = json!({
            "endpoint": endpoint,
            "method": method,
            "details": details
        });
        self.log(LogLevel::ApiCall, "api_call_start", Some(data));
    }

    /// Log API responses with timing and status
    pub fn log_api_response(&self, endpoint: &str, status: &str, duration_ms: f64, details: Option<serde_json::Value>) {
        let data = json!({
            "endpoint": endpoint,
            "status": status,
            "duration_ms": duration_ms,
            "details": details
        });
        self.log(LogLevel::ApiCall, "api_call_complete", Some(data));
    }

    /// Log function executions
    pub fn log_function_call(&self, function_name: &str, args: &serde_json::Value, details: Option<serde_json::Value>) {
        let data = json!({
            "function": function_name,
            "arguments": args,
            "details": details
        });
        self.log(LogLevel::FunctionCall, "function_call_start", Some(data));
    }

    /// Log function completion
    pub fn log_function_complete(&self, function_name: &str, duration_ms: f64, result: &serde_json::Value) {
        let data = json!({
            "function": function_name,
            "duration_ms": duration_ms,
            "result": result
        });
        self.log(LogLevel::FunctionCall, "function_call_complete", Some(data));
    }

    /// Log performance metrics
    pub fn log_performance(&self, metric: &str, value: f64, unit: &str, details: Option<serde_json::Value>) {
        let data = json!({
            "metric": metric,
            "value": value,
            "unit": unit,
            "details": details
        });
        self.log(LogLevel::Performance, "performance_metric", Some(data));
    }

    /// Log errors with stack traces if available
    pub fn log_error(&self, error: &str, context: Option<serde_json::Value>) {
        let data = json!({
            "error": error,
            "context": context,
            "stack": format!("{:?}", std::backtrace::Backtrace::capture())
        });
        self.log(LogLevel::Error, "error_occurred", Some(data));
    }

    /// Log warnings
    pub fn log_warning(&self, warning: &str, context: Option<serde_json::Value>) {
        let data = json!({
            "warning": warning,
            "context": context
        });
        self.log(LogLevel::Warn, "warning", Some(data));
    }

    /// Log state changes
    pub fn log_state_change(&self, state_type: &str, old_value: &serde_json::Value, new_value: &serde_json::Value) {
        let data = json!({
            "state_type": state_type,
            "old_value": old_value,
            "new_value": new_value
        });
        self.log(LogLevel::Info, "state_change", Some(data));
    }
}

/// Performance timer for measuring operation durations
pub struct PerformanceTimer {
    logger: ProcessLogger,
    operation: String,
    start_time: f64,
}

impl PerformanceTimer {
    pub fn start(logger: ProcessLogger, operation: &str) -> Self {
        let timer = Self {
            logger,
            operation: operation.to_string(),
            start_time: js_sys::Date::now(),
        };
        
        timer.logger.log(LogLevel::Performance, 
            &format!("{}_start", operation), 
            Some(json!({"start_time": timer.start_time})));
        
        timer
    }

    pub fn checkpoint(&self, checkpoint: &str, details: Option<serde_json::Value>) {
        let duration = js_sys::Date::now() - self.start_time;
        let data = json!({
            "checkpoint": checkpoint,
            "duration_ms": duration,
            "details": details
        });
        self.logger.log(LogLevel::Performance, 
            &format!("{}_checkpoint", self.operation), 
            Some(data));
    }

    pub fn finish(self, details: Option<serde_json::Value>) -> f64 {
        let duration = js_sys::Date::now() - self.start_time;
        let data = json!({
            "duration_ms": duration,
            "details": details
        });
        self.logger.log(LogLevel::Performance, 
            &format!("{}_complete", self.operation), 
            Some(data));
        duration
    }
}

/// Session-specific logger that tracks conversation flow
#[derive(Clone)]
pub struct SessionLogger {
    session_id: String,
    logger: ProcessLogger,
    message_count: usize,
    function_call_count: usize,
    api_call_count: usize,
    total_response_time: f64,
}

impl SessionLogger {
    pub fn new(session_id: &str) -> Self {
        let logger = ProcessLogger::new("Session").with_session(session_id);
        logger.log_user_action("session_started", Some(json!({
            "session_id": session_id
        })));
        
        Self {
            session_id: session_id.to_string(),
            logger,
            message_count: 0,
            function_call_count: 0,
            api_call_count: 0,
            total_response_time: 0.0,
        }
    }

    pub fn log_message_sent(&mut self, content: &str, message_type: &str) {
        self.message_count += 1;
        self.logger.log_user_action("message_sent", Some(json!({
            "message_count": self.message_count,
            "message_type": message_type,
            "content_length": content.len(),
            "content_preview": if content.len() > 100 { 
                format!("{}...", &content[..100]) 
            } else { 
                content.to_string() 
            }
        })));
    }

    pub fn log_api_request(&mut self, provider: &str, model: &str, request_details: Option<serde_json::Value>) {
        self.api_call_count += 1;
        self.logger.log_api_call(&format!("{}/{}", provider, model), "POST", Some(json!({
            "api_call_count": self.api_call_count,
            "provider": provider,
            "model": model,
            "request_details": request_details
        })));
    }

    pub fn log_api_response(&mut self, provider: &str, model: &str, duration_ms: f64, success: bool, details: Option<serde_json::Value>) {
        self.total_response_time += duration_ms;
        let avg_response_time = self.total_response_time / self.api_call_count as f64;
        
        self.logger.log_api_response(&format!("{}/{}", provider, model), 
            if success { "success" } else { "error" }, 
            duration_ms, 
            Some(json!({
                "success": success,
                "avg_response_time": avg_response_time,
                "total_calls": self.api_call_count,
                "details": details
            })));
    }

    pub fn log_function_execution(&mut self, function_name: &str, duration_ms: f64, success: bool, result_details: Option<serde_json::Value>) {
        self.function_call_count += 1;
        self.logger.log_function_complete(function_name, duration_ms, &json!({
            "success": success,
            "function_call_count": self.function_call_count,
            "result_details": result_details
        }));
    }

    pub fn get_session_stats(&self) -> serde_json::Value {
        json!({
            "session_id": self.session_id,
            "message_count": self.message_count,
            "function_call_count": self.function_call_count,
            "api_call_count": self.api_call_count,
            "avg_response_time": if self.api_call_count > 0 { 
                self.total_response_time / self.api_call_count as f64 
            } else { 
                0.0 
            }
        })
    }

    pub fn log_session_summary(&self) {
        self.logger.log(LogLevel::Info, "session_summary", Some(self.get_session_stats()));
    }
}

/// Global performance monitor for application-wide metrics
pub struct PerformanceMonitor;

impl PerformanceMonitor {
    /// Log memory usage if available
    pub fn log_memory_usage() {
        let logger = ProcessLogger::new("PerformanceMonitor");
        logger.log_performance("memory_check", js_sys::Date::now(), "timestamp", Some(json!({
            "note": "Memory monitoring attempted - Performance API may not be available in WASM environment"
        })));
    }

    /// Log navigation timing metrics
    pub fn log_navigation_timing() {
        let logger = ProcessLogger::new("PerformanceMonitor");
        logger.log_performance("navigation_timing", js_sys::Date::now(), "timestamp", Some(json!({
            "note": "Navigation timing attempted - Performance API may not be available in WASM environment"
        })));
    }

    /// Start monitoring resource usage
    pub fn start_resource_monitoring() {
        let logger = ProcessLogger::new("PerformanceMonitor");
        logger.log(LogLevel::Info, "resource_monitoring_started", Some(json!({
            "note": "Resource monitoring simplified for WASM environment",
            "timestamp": js_sys::Date::now()
        })));
    }
}

/// Convenience macros for common logging patterns
#[macro_export]
macro_rules! log_user_action {
    ($session_id:expr, $action:expr) => {
        $crate::llm_playground::logging::ProcessLogger::new("UserAction")
            .with_session($session_id)
            .log_user_action($action, None)
    };
    ($session_id:expr, $action:expr, $data:expr) => {
        $crate::llm_playground::logging::ProcessLogger::new("UserAction")
            .with_session($session_id)
            .log_user_action($action, Some($data))
    };
}

#[macro_export]
macro_rules! log_performance {
    ($component:expr, $metric:expr, $value:expr, $unit:expr) => {
        $crate::llm_playground::logging::ProcessLogger::new($component)
            .log_performance($metric, $value, $unit, None)
    };
    ($component:expr, $metric:expr, $value:expr, $unit:expr, $details:expr) => {
        $crate::llm_playground::logging::ProcessLogger::new($component)
            .log_performance($metric, $value, $unit, Some($details))
    };
}

#[macro_export]
macro_rules! log_error {
    ($component:expr, $error:expr) => {
        $crate::llm_playground::logging::ProcessLogger::new($component)
            .log_error($error, None)
    };
    ($component:expr, $error:expr, $context:expr) => {
        $crate::llm_playground::logging::ProcessLogger::new($component)
            .log_error($error, Some($context))
    };
}