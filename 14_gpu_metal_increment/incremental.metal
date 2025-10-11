#include <metal_stdlib>
using namespace metal;

kernel void increment(device int *data [[buffer(0)]],
                      uint count [[threads_per_grid]],
                      uint tid [[thread_position_in_grid]]) {
    if (tid < count)
        data[tid] += 1;
}
