<p align="center">
    <a href="https://github.com/orhun/kmon">
        <img src="https://user-images.githubusercontent.com/24392180/73918056-d5c45500-48d1-11ea-8d18-9943827ab2ed.png" width="500"></a>
    <br>
    <b>Linux Kernel Manager and Activity Monitor 🐧💻</b>
    <br>
    <br>
    <a href="https://github.com/orhun/kmon/actions?query=workflow%3A%22Continuous+Integration%22"><img src="https://img.shields.io/github/workflow/status/orhun/kmon/Continuous Integration/master?color=000000&label=CI&style=flat-square"></a>
    <a href="https://github.com/orhun/kmon/actions?query=workflow%3A%22Continuous+Deployment%22"><img src="https://img.shields.io/github/workflow/status/orhun/kmon/Continuous Deployment/master?color=000000&label=CD&style=flat-square"></a>
    <br>
    <a href="https://github.com/orhun/kmon/releases"><img src="https://img.shields.io/github/v/release/orhun/kmon?color=000000&style=flat-square"></a>
    <a href="https://crates.io/crates/kmon/"><img src="https://img.shields.io/crates/v/kmon?color=000000&style=flat-square"></a>
    <a href="https://aur.archlinux.org/packages/kmon/"><img src="https://img.shields.io/aur/version/kmon?color=000000&style=flat-square"></a>
    <br>
    <a href="https://codecov.io/gh/orhun/kmon"><img src="https://img.shields.io/codecov/c/gh/orhun/kmon?color=000000&style=flat-square"></a>
    <a href="https://github.com/orhun/kmon/blob/master/LICENSE"><img src="https://img.shields.io/crates/l/kmon?color=000000&style=flat-square"></a>
</p>

The kernel is the part of the operating system that facilitates interactions between hardware and software components. On most systems, it is loaded on startup after the bootloader and handles I/O requests as well as peripherals like keyboards, monitors, network adapters, and speakers. Typically, the kernel is responsible for memory management, process management, device management, system calls, and security.  
Applications use the system call mechanism for requesting a service from the operating system and most of the time, this request is passed to the kernel using a library provided by the operating system to invoke the related kernel function. While the kernel performs these low-level tasks, it's resident on a separate part of memory named protected kernel space which is not accessible by applications and other parts of the system. In contrast, applications like browsers, text editors, window managers or audio/video players use a different separate area of the memory, user space. This separation prevents user data and kernel data from interfering with each other and causing instability and slowness, as well as preventing malfunctioning application programs from crashing the entire operating system.  
There are different kernel designs due to the different ways of managing system calls and resources. For example, while monolithic kernels run all the operating system instructions in the same address space for speed, microkernels use different spaces for user and kernel services for modularity. Apart from those, there are hybrid kernels, nanokernels, and, exokernels. The hybrid kernel architecture is based on combining aspects of microkernel and monolithic kernels.

The Linux kernel is the open-source, monolithic and, Unix-like operating system kernel that used in the Linux distributions, various embedded systems such as routers and as well as in the all Android-based systems. Linus Torvalds conceived and created the Linux kernel in 1991 and it's still being developed by thousands of developers today. It's a prominent example of free and open source software and it's used in other free software projects, notably the GNU operating system.
Although the Linux-based operating systems dominate the most of computing, it still carries some of the design flaws which were quite a bit of debate in the early days of Linux. For example, it has the largest footprint and the most complexity over the other types of kernels. But it's a design feature that monolithic kernels inherent to have. These kind of design issues led developers to add new features and mechanisms to the Linux kernel which other kernels don't have.

Unlike the standard monolithic kernels, the Linux kernel is also modular, accepting loadable kernel modules (LKM) that typically used to add support for new hardware (as device drivers) and/or filesystems, or for adding system calls. Since LKMs could be loaded and unloaded to the system at runtime, they have the advantage of extending the kernel without rebooting and re-compiling. Thus, the kernel functionalities provided by modules would not reside in memory without being used and the related module can be unloaded in order to free memory and other resources.

## Installation

## Usage

### Key Bindings

### Command Line Arguments

## Examples

## Docker

## TODO(s)

## Contributing

## License

GNU General Public License ([3.0](https://www.gnu.org/licenses/gpl.txt))

## Copyright

Copyright (c) 2020, [orhun](mailto:orhunparmaksiz@gmail.com)