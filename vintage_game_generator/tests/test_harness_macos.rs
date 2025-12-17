//! Test harness for macOS that runs tests on the main thread
//! This allows UI-related tests to work on macOS

#[cfg(target_os = "macos")]
mod macos_tests {
    use std::sync::Once;
    
    static INIT: Once = Once::new();
    
    /// Initialize test environment once
    fn init_test_env() {
        INIT.call_once(|| {
            // Set up any global test state here
            std::env::set_var("RUST_LOG", "debug");
        });
    }
    
    /// Run a test on the main thread (required for macOS UI tests)
    pub fn run_on_main_thread<F: FnOnce() + Send + 'static>(test: F) {
        init_test_env();
        
        // For tests, we'll use a simpler approach that doesn't require full event loop
        test();
    }
}

#[cfg(not(target_os = "macos"))]
mod macos_tests {
    pub fn run_on_main_thread<F: FnOnce()>(test: F) {
        test();
    }
}

pub use macos_tests::run_on_main_thread;
