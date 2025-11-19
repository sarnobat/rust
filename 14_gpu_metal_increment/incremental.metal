// Metal Shading Language (MSL), which is essentially C++-like syntax

#include <metal_stdlib>
using namespace metal;

kernel void increment(
    device int *data [[buffer(0)]],      // pointer to data buffer in GPU memory
    uint count [[threads_per_grid]],    // total number of threads (data length)
    uint tid [[thread_position_in_grid]]) {  // unique thread ID across the grid


    if (tid < count)           // Ensure this thread index is within the data array bounds
    {                          
        data[tid] =            // Write the result back into GPU data buffer
            (data[tid] * 3     // 1. Multiply current element by 3 (scales the value)
            + 7)               // 2. Add 7 (adds an offset)
            ^                  // 3. XOR with the next expression (bitwise mix)
            (data[tid] >> 2);  // 4. Shift original value right by 2 bits (divide by 4, drop low bits)
    }

}
