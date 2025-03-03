# Commit Log (15 marks)
7.5 marks for each implementation's commit log

1.  Pull request title (0.5 marks)
    * Explains the changes
    * Less than 80 characters

2.  Summary (1 mark)
    * Explains the changes

3.  Tech details (1.5 marks)
    * Explains what changes were made and why

4.  Correctness (1.5 marks)
    * Explains how the code was tested for correctness (e.g. unit tests, end-to-end tests with comparison tool)

5.  Performance (1.5 marks)
    * Explains how the code was tested for performance (e.g. hyperfine, profiling)

6.  Clarity of exposition (1.5 marks)
    1. Clear explanation (0.5 marks)
    2. Formatting / grammar (0.5 marks)
        * no significant readability issues
    3. Length (0.5 marks)
        * not too long

# Code Implementation (85 marks)

## Separate maps (40 marks)
    1. single-threaded correctness (8 tests x 1 mark each = 8 marks)
        * 'Test {1-8}': expect an exact match with the original output

    2. multi-threaded correctness (6 tests x 4 marks each = 24 marks)
        * 'Test {9,10}' with {2, 4, 8} threads; match within tolerance

    3. num_threads (2 tests x 2 mark each = 4 mark)
        * Number of created threads must be proportional to num_thread argument

    4. num_threads (4 mark)
        * Test {0}  num_thread

## Concurrent maps (45 marks)
    1. single-threaded correctness (8 tests x 1 mark each = 8 marks)
        * 'Test {1-8}': expect an exact match with the original output

    2. multi-threaded correctness (6 tests x 4 marks each = 24 marks)
        * 'Test {9,10}' with {2, 4, 8} threads; match within tolerance

    3. speedup (2 tests x 3 marks each = 6 marks)
        * 'Test {9,10}' with {16} threads; expect at least 5% speedup

    4. num_threads (2 tests x 2 mark each = 4 mark)
        * Number of created threads must be proportional to num_thread argument
    
    5. num_threads (3 mark)
        * Test {0}  num_thread

## Why is multi-threaded correctness 4 marks?
    1.  dict headers within tolerance: 0.5 marks
    2.  double dict within tolerance: 1 mark
    3.  triple dict within tolerance: 1 mark
    4.  2-grams within tolerance: 0.5 marks
    5.  sample token list diff size within tolerance: 0.5 marks
    6.  dynamic token list diff size within tolerance: 0.5 marks