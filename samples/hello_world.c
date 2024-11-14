#include <stdint-gcc.h>

uint8_t array[32] = {'H', 'e', 'l', 'l', 'o', ' ', 'W', 'o', 'r', 'l', 'd', '!', '\n', '\0'};
uint32_t myvalue = 0x12345678;
uint32_t myvalue2 = 0xAA55AA55;
uint32_t myvalue3 = 1;
uint32_t myvalue4 = 2;
uint32_t myvalue5 = 3;
uint32_t myvalue6 = 4;

uint8_t __attribute__((section(".bss"))) bss[16];
uint32_t __attribute__((section(".noinit"), used)) noinitvar[2];

static int32_t __syscall_test(uint32_t r0, uint32_t r1, uint32_t r2, uint32_t r3)
{
    __asm__ __volatile__(
        "mov r0, %0\n"
        "mov r1, %1\n"
        "mov r2, %2\n"
        "mov r3, %3\n"
        "svc 0\n"
        :
        : "r"(r0), "r"(r1), "r"(r2), "r"(r3)
        : "memory"
    );

    return (int32_t)r0;
}

static int32_t __syscall_io(uint32_t r0, uint32_t r1, uint32_t r2, uint32_t r3)
{
    __asm__ __volatile__(
        "mov r0, %0\n"
        "mov r1, %1\n"
        "mov r2, %2\n"
        "mov r3, %3\n"
        "svc 2\n"
        :
        : "r"(r0), "r"(r1), "r"(r2), "r"(r3)
        : "memory"
    );

    return (int32_t)r0;
}

static int32_t __syscall_kernel(uint32_t r0, uint32_t r1, uint32_t r2, uint32_t r3)
{
    __asm__ __volatile__(
        "mov r0, %0\n"
        "mov r1, %1\n"
        "mov r2, %2\n"
        "mov r3, %3\n"
        "svc 1\n"
        :
        : "r"(r0), "r"(r1), "r"(r2), "r"(r3)
        : "memory"
    );

    return (int32_t)r0;
}

#define k_yield() __syscall_kernel(0, 0, 0, 0)
#define k_sleep(ms) __syscall_kernel(ms, 0, 0, 1)

static uint32_t myfunc(uint32_t r0)
{
    return r0;
}

int _start(void *arg)
{
    array[0] = 'B';
    myvalue = 0x87654321;
    myvalue = myfunc(myvalue);
    myvalue2 = myfunc(myvalue2);
    myvalue3 = myfunc(myvalue3);
    noinitvar[0] = myfunc(noinitvar[1]);


    int32_t ret = __syscall_kernel(10, 0, 0, 8);
    __syscall_test((uint32_t)ret, 0, 0, 0);
    k_sleep(100);
    __syscall_io((uint32_t)array, 32, 0, 10);
    k_sleep(100);
    __syscall_io((uint32_t)array, 32, 0, 0);
    k_sleep(100);
    __syscall_test(myvalue3, myvalue4, myvalue5, myvalue6);
    k_sleep(100);
    __syscall_test((uint32_t)arg, (uint32_t)array, 32, myvalue);
    k_sleep(100);

    myvalue3 = 0;
    myvalue4 = 0;
    myvalue5 = 0;
    myvalue6 = 0;

    __syscall_test(myvalue3, myvalue4, myvalue5, myvalue6);

    return 42;
}
