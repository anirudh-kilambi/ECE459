# Final CUDA Implementation

## Summary
No updates were required in `main.rs`, `cnn.rs`, and `cpu.rs`. Additions were made to `kernel.cu` and `cuda.rs` to implement the CUDA version of the CNN. 
There are three major computation components to the CNN, the *convolution layer*, *ReLU layer*, and *output layer*. This computation was being moved from within
the Rust code and placed inside the kernel file. `cuda.rs` required two implementations, one for the initialization of the CUDA context, and the other for the 
invocations of the kernel function. 

## Technical Details
The `init` function in `cuda.rs` was used to initialize the CUDA context. This function retrieved the data from `kernel.ptx`, and loaded it into the CudaContext.

`kernel.cu` required the implementation of the three major computation components. The convolution layer was implemented using the `conv_layer` function. 
The ReLU layer was implemented using the `relu` function, and the output layer was implemented using the `output` function. Throughout all the kernel functions, 
an important concept was the fact data wasn't being returned from the functions, rather, whatever needed to be returned was passed in as an empty data structure 
that could be modified from within the kernel. These data structures were shared from the Rust file, `cuda.rs` to the kernel. Another important concept was the idea 
of defining the block, grid, and thread sizes. These were based off the dimensions of the input data and the size of the filter. In this case, since dot products are computed,
this helps alleviate potential optimizations that might be required for these dimensions. The convolutional layer takes the input matrix of the image, and computes the dot product with the 
filter matrix, which has 10 5x5 filters (for 10 neurons). The dot product of each 5x5 submatrix of the input was computed with each of the filters, this yields an output matrix of shape 10x20x20. 
In this case, three data structures were needed for computation, or manipulation -- the input matrix, the convolutional filters, and the output matrix. The output matrix was initialized to zeros, 
and populated with values from the `conv_layer` kernel. The block size of the `conv_layer` kernel was defined as (1, 1, 10). This would allow for iterating through each of the individual 5x5 filters 
for each neuron. The grid size, was used to iterate through the 5x5 sub matrices. Since the input matrix had a shape of 100x100, this means that we would need to iterate through 20 5x5 sub matrices 
in the x and y directions. As a result, the grid size was selected to be (20, 20). This leaves the x,y directions of the matrix to be on the builtin `Idx.x` and `Idx.y` variables. 

The ReLU layer only handled the changing of negative values to zero. This was done on the output matrix of the convolutional layer. That means the grid and block size of the ReLU kernel exactly matched the 
`conv_layer` kernel. Since changes could be done within the matrix being passed in (the output of the `conv_layer` kernel), no additional data structures were needed. The only input of the ReLU kernel was the output matrix.
This was simply changed within the kernel and "returned". 

The `output` kernel was used to the compute the dot product between the output of the `relu` kernel and the weights. The weights was a 10x4000 matrix (4000 weights per neuron). The output would be an array of 10 scalars.
Since there were 10 neurons, the block size was as (1, 1, 1), while the grid size was set as (10, 1, 1). This means that the dot product would be computed for each neuron. This means that there would be 10 invocations of the kernel overall,
each one doing 4000 the computation of 4000 values. The input to the `output` kernel was the output matrix of the `relu` kernel, the weights matrix, and an empty output array. As dot products were computed (per neuron), the output array was updated.

## Evaluating Correctness
To evaluate the correctness of the CUDA implementation, the output of the CUDA implementation was compared to the output of the CPU implementation. Multiple input images were used, and no issues were found when using `compare.py`.

## Evaluating Performance
The performance of the CPU implementation took on average 20000 microseconds to compute the output of the CNN. The CUDA implementation took an average of 55000 microseconds. This was unexpected, as I thought the CUDA implementatiom should be faster. 
Common points of overhead include unnecessary extra invocations of kernels, unnecessary memory transfers, unnecessary conditionals, and innefficient block and grid sizing. The only data structures transferred to the GPU memory were required for computation, 
and only a single conditional was used in the ReLU kernel, which was necessary. However, I believe the bottleneck in the CUDA programming occurs at the output layer. Because I am only allowing for 10 invocations of the `output` kernel, this means each kernel is 
computing 4000 loops. This might be a lot of computation for each kernel, and might be the reason for the slowdown. Perhaps using a grid size of (10, 10), reducing the number of computations to 400 per kernel would provide speedups. However, since we are now 
computing parts of an individual output in parallel, this might require the use of non-blocking streams. Since all we are doing is a sum, this might be unnecessary. 

