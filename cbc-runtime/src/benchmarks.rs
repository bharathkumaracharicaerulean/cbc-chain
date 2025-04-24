// This file is part of the CBC Runtime.
// It defines the benchmarking configuration for various FRAME pallets used in the runtime.

// The macro `frame_benchmarking::define_benchmarks!` is used to register all the benchmarking
// test modules that will be executed during runtime performance measurement.
// Each entry represents a benchmarking target associated with a specific pallet/module.

frame_benchmarking::define_benchmarks!(
    // This line registers the benchmark module defined in `frame_benchmarking`, under `BaselineBench`,
    // which provides baseline benchmarks such as block execution time, extrinsics, etc.
    [frame_benchmarking, BaselineBench::<Runtime>]

    // Benchmarks for the `frame_system` pallet, which handles core blockchain functionality
    // such as block production, account management, and extrinsic validation.
    // `SystemBench` contains performance tests specific to the system operations.
    [frame_system, SystemBench::<Runtime>]

    // Benchmarks for `frame_system_extensions`, possibly a custom extension or wrapper around system pallet logic.
    // These benchmarks might involve extended system-level functionalities like custom hooks or weight calculations.
    [frame_system_extensions, SystemExtensionsBench::<Runtime>]

    // Benchmarks for the `pallet_balances`, which manages the account balance information.
    // It includes benchmarking of functions like transfers, setting balances, reserving funds, etc.
    [pallet_balances, Balances]

    // Benchmarks for `pallet_timestamp`, responsible for setting the on-chain timestamp.
    // Measures performance of time-setting and any time-dependent logic.
    [pallet_timestamp, Timestamp]

    // Benchmarks for the `pallet_sudo`, which allows privileged access to call any function.
    // These tests measure overhead and weight of administrative operations through sudo.
    [pallet_sudo, Sudo]

    // Benchmarks for a custom or template pallet called `pallet_template`.
    // This is typically a starting point for developing custom pallets and includes benchmarking
    // for any custom extrinsics or storage operations defined within the template.
    [cbc_pallet_template, Template]
);
