/*
 * Copyright (c) 2024 Lucas Dietrich <ld.adecy@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

#include <libc.h>
#include <libuser.h>
#include <helpers.h>
#include <stdint-gcc.h>

uint8_t array[32] = {'H', 'e', 'l', 'l', 'o', ' ',	'W',
					 'o', 'r', 'l', 'd', '!', '\n', '\0'};
uint32_t myvalue  = 0x12345678;
uint32_t myvalue2 = 0xAA55AA55;
uint32_t myvalue3 = 1;
uint32_t myvalue4 = 2;
uint32_t myvalue5 = 3;
uint32_t myvalue6 = 4;

uint8_t __attribute__((section(".bss"))) bss[16];
uint32_t __attribute__((section(".noinit"), used)) noinitvar[2];

static uint32_t myfunc(uint32_t r0)
{
	return r0;
}

// move _start to libc.c, keep main in hello_world.c
int _start(void *arg)
{
	array[0]	 = 'B';
	myvalue		 = 0x87654321;
	myvalue		 = myfunc(myvalue);
	myvalue2	 = myfunc(myvalue2);
	myvalue3	 = myfunc(myvalue3);
	noinitvar[0] = myfunc(noinitvar[1]);

	int32_t ret = __syscall_kernel(10, 0, 0, 8);
	k_syscall_test((uint32_t)ret, 0, 0, 0);
	k_sleep(100);
	k_syscall_test((uint32_t)array, 32, 0, 10);
	k_sleep(100);
	k_syscall_test((uint32_t)array, 32, 0, 0);
	k_sleep(100);
	k_syscall_test(myvalue3, myvalue4, myvalue5, myvalue6);
	k_sleep(100);
	k_syscall_test((uint32_t)arg, (uint32_t)array, 32, myvalue);
	k_sleep(100);

	myvalue3 = 0;
	myvalue4 = 0;
	myvalue5 = 0;
	myvalue6 = 0;

	uint32_t value = 0x12345678;

	static const char *str = "AAASTATIC CONST CHAR*";
	int len = strlen(str);
	int z = k_syscall_test(len, 1, 0, 0);

	if (z) value++;

	const char *str2 = "AAACONST CHAR*";
	len = strlen(str2);
	k_syscall_test(len, 2, 0, value);

	char *str3 = "AAACHAR*";
	len = strlen(str3);
	k_syscall_test(len, 3, 0, 0);

	str3 = "AAACHAR*B";
	len = strlen(str3);
	k_syscall_test(len, 3, 0, 0);

	str3 = "AAACHAR*BC";
	len = strlen(str3);
	k_syscall_test(len, 3, 0, 0);

	k_syscall_test(myvalue3, myvalue4, myvalue5, myvalue6);

	return 42;
}
