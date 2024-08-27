/* Priority of these interruptions cannot be changed */
pub const NONMASKABLEINT: i32 = -14; /*  2 Non Maskable Interrupt */
pub const HARDFAULT: i32 = -13; /*  3 HardFault Interrupt */

/*
 * Priorities of these system interrupts can be changed (v7-M Architecture Reference Manual):
 * B3.2.12 System Handler Priority Register 3, SHPR3
 *
 * +-----------+-----------+-----------+-----------+
 * |   31-24   |   23-16   |   15-8    |   7-0     |
 * +===========+===========+===========+===========+
 * |  PRI_15   |  PRI_14   |  PRI_13   |  PRI_12   |
 * +-----------+-----------+-----------+-----------+
 *
 * - **PRI_15**, bits[31:24]: Priority of system handler 15, SysTick.
 * - **PRI_14**, bits[23:16]: Priority of system handler 14, PendSV.
 * - **PRI_13**, bits[15:8]: Reserved for priority of system handler 13.
 * - **PRI_12**, bits[7:0]: Priority of system handler 12, DebugMonitor.
 *
 */
#[repr(i32)]
pub enum SysIrqn {
    MEMORYMANAGEMENT = -12, /*  4 Memory Management Interrupt */
    BUSFAULT = -11,         /*  5 Bus Fault Interrupt */
    USAGEFAULT = -10,       /*  6 Usage Fault Interrupt */
    SVCALL = -5,            /* 11 SV Call Interrupt */
    DEBUGMONITOR = -4,      /* 12 Debug Monitor Interrupt */
    PENDSV = -2,            /* 14 Pend SV Interrupt */
    SYSTICK = -1,           /* 15 System Tick Interrupt */
}
