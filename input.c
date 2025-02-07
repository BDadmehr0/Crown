#include <stdio.h>
#include <stdlib.h>

// Pointer Usage
void pointer_example() {
    int a = 10;
    int* ptr = &a;
    printf("%d\n", *ptr);
}

// Dynamic Memory Allocation
void memory_example() {
    int* arr = (int*)malloc(5 * sizeof(int));
    for (int i = 0; i < 5; i++) {
        arr[i] = i * 2;
        printf("%d ", arr[i]);
    }
    free(arr);
}

// Struct Example
struct Point {
    int x;
    int y;
};

// Function Example
void function_example(struct Point p) {
    printf("Point(%d, %d)\n", p.x, p.y);
}

int main() {
    pointer_example();
    memory_example();
    struct Point p = {10, 20};
    function_example(p);
    return 0;
}
