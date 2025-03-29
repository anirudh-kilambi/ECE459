// Very minimal skeleton for the kernel

#include <stdio.h>


// Definition of conv_layer Kernel
//dim3 grid(20,20);
//dim3 block(5,5);
extern "C" __global__ void conv_layer (
        double input_matrix[100][100],
        double conv_filters[10][5][5],
        double conv_output[10][20][20]
        ) {
    // need to define our loops.
    // We'll have 5x5 blocks going through the input_matrix
    // The 5x5 blocks of the input_matrix will have 
    // the dot product computed with the filter, there are 10 5x5 filters.
    // In this case, blockIdx.x and blockIdx.y gives which 5x5 sub-matrix 
    // blockDim.x and blockDim.y gives the size of each block
    // threadIdx.x and threadIdx.y gives the iterator inside each block in the i, j directions

    // we need to get the row and column starting points. This will get a threadId value passed into it
    // to receive the global thread ID, this is based off the global thread ID.
    // but for this case, we can pull the value from an iterator between 0, 5
    // sizes have been defined as y representing the row (0 - 19), z representing the column (0-19), x represents the filter index,
    // 0 - 9
    // Thread Id represents the i or j value (0 - 4)
    int row_global_thread_id = blockIdx.y * blockDim.y + threadIdx.y;
    int col_global_thread_id = blockIdx.z * blockDim.z + threadIdx.z;
    int row = row_global_thread_id * 5;
    int col = col_global_thread_id * 5;
    int filter = blockIdx.x;
    // this is the dot product for a given matrix, initialized as 0
    double dot_prod = 0.0; 


    for (int y = 0; y < 5; y++) {
        for (int x = 0; x < 5; x++) {
            dot_prod += input_matrix[row + y][col + x] * conv_filters[filter][y][x];
        }
    }
    // each 5x5 at some blockId returns 1 value (dot product returns a scalar)
    // that means that the dot product of a sub-matrix corresponds to the blockId's index value in the output
    conv_output[filter][row_global_thread_id][col_global_thread_id] = dot_prod;

}

extern "C" __global__ void relu (double conv_output[10][20][20]) {
    int row = blockIdx.y * blockDim.y + threadIdx.y;
    int col = blockIdx.z * blockDim.z + threadIdx.z;
    int filter = blockIdx.x;

    if (conv_output[filter][row][col] < 0) {
        conv_output[filter][row][col] = 0;
    }

}

extern "C" __global__ void output (
        double relu_output[10][20][20], // this is just the ReLU output, a version of conv_output
        double weights[10][4000],
        double output[10] // the output is just a matrix of 4000x1
        ) {
    // we have 10 neurons, so need to determine which neuron we are computing
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    int neuron_idx = 0;
    double dot_prod = 0.0;

    for (int z = 0; z < 10; z++) {
        for (int y = 0; y < 20; y++) {
            for (int x = 0; x < 20; x++) {
                dot_prod += relu_output[z][y][x] * weights[idx][neuron_idx];
                neuron_idx++;
            }
        }
    }
    output[idx] = dot_prod;
}




