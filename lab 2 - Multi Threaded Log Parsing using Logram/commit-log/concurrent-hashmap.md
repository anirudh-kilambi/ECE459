# Final Concurrent Hashmap Implementation 

## Summary
No updates were required in `main.rs` as that was handled during the `single-map` implementation. The logic for the concurrent hashmap was implemented in `parser.rs` as a new function called 
`dictionary_builder_concurrent_map`. The function was very similar to the `dictionary_builder_single_map` implementation, with the major change being the handling of data structures in the worker threads,
and the handling of the threads after they were joined back together. To complete this implementation, the `dashmap::DashMap` and `dashmap::DashSet` objects were used to replace the use of `HashMap` and `HashSet` objects
in the `single-map` implementation. 

## Technical Details
The pre-processing logic remained exactly the same, with the use of the `chunk_map` to determine partitioning of the log file. Additionally, the handling of `prev1` and `prev2` at the start of each thread remained the same.
The biggest change was the removal of the need to create a `dbl`, `trpl`, and `all_tokens` object in each thread. The global definition of these structures were created as `Arc<DashMap<>` and `Arc<DashSet<>>` objects. These could be
cloned after the spawning of each thread to be used by them. 

Since `process_dictionary_builder_line` handles the mutating of these data structures, a copy of the function had to be made that took in the `Arc<DashMap<>>` and `Arc<DashSet<>>` types for their respective data structures. Finally, with the use
of these dashmap data structures, there was no need to return anything from the worker threads as the thread's copies of the data structures were clones of the global data structures. However, once the threads were joined back together, the now populated
data structures needed to be converted back to HashMaps and a `Vec<String>`. This was done and the resulting data structures were returned.

## Testing for correctness
Since the partitioning logic was tested in the `single-map` implementation, the marmoset test cases were run against the concurrent solution to ensure correctness of the n-gram algorithm. The majority of the testing here had to do with performance.

## Performance
The performance of this multi-threaded implementation was measured using the `time` command, as well as the `std::time::Instant` object. The time command showed an average runtime of 8 seconds for the overall runtime with the HealthApp log, however, the use of a `DashSet` and `DashMap`
showed a 66% reduction in runtime for the `dictionary_builder` function. The `single-map` implementation took on average around 3 seconds to run, while the `concurrent-map` implementation took around 1 second to run.

Both solutions were much faster than the original `single-threaded` implementation, which took around 18 seconds to run. However, if the concurrent map solution was used in `single-threaded` mode, the runtime was around 2 seconds, while the `single-map` implementation took around 17 seconds.
One possible reason for this is the use of a set for the `all_tokens` data structure within the function. While a `Vec<String>` was still returned for the `all_tokens` data structure, the use of a set for the structure within `proces_dictionary_builder_line`, and the lack of need to ensure the vector had no duplicates
will save overhead. 

On smaller files, concurrency will show smaller improvements since each thread will have much less work to do. This does not hold for larger files were the ability to pawn the work off to multiple threads greatly improves performance.


