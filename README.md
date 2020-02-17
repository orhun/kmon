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

The kernel is the part of the operating system that facilitates interactions between hardware and software components. On most systems, it is loaded on startup after the bootloader and handles I/O requests as well as peripherals like keyboards, monitors, network adapters, and speakers. Typically, the kernel is responsible for memory management, resource management, and device management.  
Applications use the system call mechanism for requesting a service from the operating system and most of the time, this request is passed to the kernel using a library provided by the operating system to invoke the related kernel function. While the kernel performs these low-level tasks, it's resident on a separate part of memory named protected kernel space which is not accessible by applications and other parts of the system. In contrast, applications like browsers, text editors, window managers or audio/video players use a different separate area of the memory, user space. This separation prevents user data and kernel data from interfering with each other and causing instability and slowness, as well as preventing malfunctioning application programs from crashing the entire operating system.  
There are different kernel designs due to the different ways of managing system calls and resources. For example, while monolithic kernels run all the operating system instructions in the same address space for speed, microkernels use different spaces for user and kernel services for modularity.

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