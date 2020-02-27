use sc_executor::native_executor_instance;

// Declare an instance of the native executor named `Executor`. Include the wasm binary as the
// equivalent wasm code.
native_executor_instance!(
    pub Executor,
    datahighway_runtime::api::dispatch,
    datahighway_runtime::native_version,
);
