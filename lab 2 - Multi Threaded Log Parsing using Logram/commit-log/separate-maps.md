# Merge Single Map Implementation into Main 
## Summary
Had to update `main.rs` to ensure that the `--single_map` and `--num-threads` flags are passed to the variables. `--num-threads` is always passed to the `num_threads` variable as it defaults
to 8 if not passed. If `--num-threads` was set to 0, then the sequential solution was used. 

The logic for single-map was:
```
if single_map is None => single_map = false;
if single_map is Some() => Assign value based on boolean passed at command line
```

Then I implemented the single map logic in `parser.rs`. Work done here will be discussed in the next section.


## Technical Details
A goal of my implementation was to mitigate the amount of changes that needed to be done to the core parsing logic. In the best case, I wanted my thread to handle the log parsing as similarly
to the single-threaded implementation as possible (`dictionary_builder`).

When dealing with a multi-threaded implementation, the most important part is to ensure that there is not overlap in the data that is being processed. That is, each line of the log file must go
through `process_dictionary_builder_line` only once. To handle, this I made a HashMap that contained the following information:
```
{
    thread_num : (start_index, end_index, last line of previous chunk, first line of next chunk)
}
```
This hash map can be made by the main thread and should be passed to the worker threads. The first step was to determine the size of each chunk (`chunk_size` in fn `get_chunk_map`). This was done by
doing ceiling division (number of lines in log file)/(number of threads). Start indices for each thread would therefore always be `chunk_size * thread_num`. The end index was a little bit more interesting,
To ensure no overlap, and due to my desire to create an `Iterator<Lines>` object to use in each thread, I used the `.skip()` and `.take()` methods to get the correct lines. However this is inclusive, so to
ensure that no overlap arised in the iterator, whose elements would be passed to the `process_dictionary_builder_line` function, the end index would always be the start index of the next thread minus 1.

Another consideration had to do with the last line from the previous thread, and the first line of the next thread. In the single threaded implementation, this is not an issue since we parsed over the whole file
in one thread. However since the file was being partitioned, technically the end of each thread would seem like the end of the file, when in reality that is only true for the final thread. To handle this, the previous
line was taken by skipping to to the end index of the previous thread, and taking the next line. This logic was used for threads `1 -> n` (since thread 0 would not have a previous line).

The first line of the next thread was done by skipping to the end index of the current thread, and taking the next line. This logic was used for threads `0 -> (n-1)` (since thread n would not have a next line).

With a chunk map, it is possible to start working on the log parsing. `prev1` and `prev2` would need to be managed for threads `1 -> n`. Since the `chunk_map` stored the last line of the previous chunks, some logic 
was stolen from `process_dictionary_builder_line` to create these values based on the thread number.

A couple of values need to be passed to the worker threads. `prev1` and `prev2` are passed to ensure that the first line of the thread is processed correctly. The start and end indices were also passed to ensure the 
iterator was produced correctly. Finally, the look-ahead line needed to be passed to ensure that the last line of the thread was processed correctly.

After moving in the required values to the worker thread, the worker thread could create the iterator, and the logic followed very closely to the single-threaded implementation. The only difference occurred when the next line
was found to be `None`. Since this would happen at the end of each thread, but did not necessarily imply no lookahead line, so a look-ahead line was always passed to the thread, that was non-null for all threads except the last.
Since we were working with a map per thread, a new `dbl`, `trpl`, and `all_tokens` object was created in each thread to be populated. These three structures were returned in each thread handle.

## Testing for correctness
The largest pain point in this implementation was ensuring the partitioning was done correctly. To ensure the indices picked were correct, the chunk_map was created, and the output was compared against the file. That is,
based on the start and end indices, the first line of the next chunk was compared against what the indices would provide you, and similarly for the last line of the previous chunk. Finally, the implementation was tested against
the marmoset test cases for correctness, as well as the test cases provided in `parser.rs`.

## Performance
The performance of this multi-threaded implementation was measured using the `time` command, as well as the `std::time::Instant` object. The time command showed an average runtime of 9 seconds for the overall runtime with the HealthApp log
test case, and around 3 seconds for just the running of the `dictionary_builder_single_map` function.


