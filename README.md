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

**The kernel** is the part of the operating system that facilitates interactions between *hardware* and *software* components. On most systems, it is loaded on startup after the *bootloader* and handles I/O requests as well as peripherals like keyboards, monitors, network adapters, and speakers. Typically, the kernel is responsible for **memory management**, **process management**, **device management**, **system calls**, and **security**.
Applications use the **system call** mechanism for requesting a service from the operating system and most of the time, this request is passed to the kernel using a library provided by the operating system to invoke the related kernel function. While the kernel performs these low-level tasks, it's resident on a separate part of memory named **protected kernel space** which is not accessible by applications and other parts of the system. In contrast, applications like browsers, text editors, window managers or audio/video players use a different separate area of the memory, **user space**. This separation prevents user data and kernel data from interfering with each other and causing instability and slowness, as well as preventing malfunctioning application programs from crashing the entire operating system.  
There are different kernel designs due to the different ways of managing system calls and resources. For example, while **monolithic kernels** run all the operating system instructions in the same address space *for speed*, **microkernels** use different spaces for user and kernel services *for modularity*. Apart from those, there are **hybrid kernels**, **nanokernels**, and, **exokernels**. The hybrid kernel architecture is based on combining aspects of microkernel and monolithic kernels.

**The Linux kernel** is the open-source, monolithic and, Unix-like operating system kernel that used in the Linux distributions, various embedded systems such as routers and as well as in the all Android-based systems. **Linus Torvalds** conceived and created the Linux kernel in 1991 and it's still being developed by thousands of developers today. It's a prominent example of **free and open source software** and it's used in other free software projects, notably the **GNU operating system**.
Although the Linux-based operating systems dominate the most of computing, it still carries some of the design flaws which were quite a bit of debate in the early days of Linux. For example, it has the **largest footprint** and **the most complexity** over the other types of kernels. But it's a design feature that monolithic kernels inherent to have. These kind of design issues led developers to add new features and mechanisms to the Linux kernel which other kernels don't have.

Unlike the standard monolithic kernels, the Linux kernel is also **modular**, accepting **loadable kernel modules (LKM)** that typically used to add support for new *hardware* (as device drivers) and/or *filesystems*, or for adding *system calls*. Since LKMs could be loaded and unloaded to the system *at runtime*, they have the advantage of extending the kernel without rebooting and re-compiling. Thus, the kernel functionalities provided by modules would not reside in memory without being used and the related module can be unloaded in order to free memory and other resources.  
Loadable kernel modules are located in `/lib/modules` with the `.ko` (*kernel object*) extension in Linux. While the [lsmod](https://linux.die.net/man/8/lsmod) command could be used for listing the loaded kernel modules, [modprobe](https://linux.die.net/man/8/modprobe) is used for loading or unloading a kernel module.

Here's a simple example of a Linux kernel module that prints a message when it's loaded and unloaded. The build and installation steps of the [module](https://github.com/orhun/kmon/blob/master/example/lkm_example.c) using a [Makefile](https://github.com/orhun/kmon/blob/master/example/Makefile) are shown below.

```
make                         # build
sudo insmod lkm_example.ko   # install
sudo modprobe lkm_example    # load
sudo modprobe -r lkm_example # unload
```

![dmesg output](https://user-images.githubusercontent.com/24392180/74931125-0dfa8600-53f0-11ea-8037-60024564ad3d.png)

The [dmesg](https://linux.die.net/man/8/dmesg) command is used above to retrieve the message buffer of the kernel.

## Installation

### Cargo

**kmon** can be installed from [crates.io](https://crates.io/crates/kmon/) using Cargo if [Rust](https://www.rust-lang.org/tools/install) is installed.

```
cargo install kmon
```

Use the `--force` option to update.

```
cargo install kmon --force
```

### AUR

**kmon** can be installed from available [AUR packages](https://aur.archlinux.org/packages/?O=0&SeB=nd&K=Linux+kernel+manager+and+activity&outdated=&SB=n&SO=a&PP=50&do_Search=Go) using an [AUR helper](https://wiki.archlinux.org/index.php/AUR_helpers). For example,

```
trizen kmon
```

### Manual

1. Download the latest binary from [releases](https://github.com/orhun/kmon/releases).

```
wget https://github.com/orhun/kmon/releases/download/v[VERSION]/kmon-[VERSION].tar.gz
```
2. Extract the files.

```
tar -xvzf kmon-*.tar.gz
```

3. Run the binary.

```
./kmon
```

4. Move binary to `/usr/local/bin/` for running it from the terminal using `kmon` command. (Optional)

#### Note

[libxcb](https://xcb.freedesktop.org/) should be installed for using the copy/paste commands of X11.
[*](https://github.com/aweinstock314/rust-clipboard/issues/67)

## Usage

```
kmon [FLAGS] [OPTIONS] [SUBCOMMANDS]
```

### Flags

```
-h, --help       Prints help information
-r, --reverse    Reverse the kernel module list
-u, --unicode    Show Unicode symbols for the block titles
-V, --version    Prints version information
```

### Options

```
-c, --color <COLOR>    Set the main color using hex or color name [default: darkgray]
-t, --tickrate <MS>    Set the refresh rate of the terminal [default: 250]
```

### Subcommands

```
help    Prints this message or the help of the given subcommand(s)
sort    Sort kernel modules
```

```
kmon sort [FLAGS]

FLAGS:
    -n, --name    Sort modules by their names
    -s, --size    Sort modules by their sizes
```

## Key Bindings

Press '`?`' while running the terminal UI to see key bindings.

### Examples

## Docker

[![Docker Hub Build Status](https://img.shields.io/docker/cloud/build/orhunp/kmon?color=000000&label=docker%20hub&style=flat-square)](https://hub.docker.com/r/orhunp/kmon)  [![Package Registry Build Status](https://img.shields.io/docker/cloud/build/orhunp/kmon?color=000000&label=package%20registry&style=flat-square)](https://github.com/orhun/kmon/packages/95852)

```
docker run -it --cap-add syslog orhunp/kmon:tagname
```
### Build

```
docker build -t kmon .
```

### Run

```
docker run -it --cap-add syslog kmon
```

## Contributing

## License

GNU General Public License ([3.0](https://www.gnu.org/licenses/gpl.txt))

## Copyright

Copyright (c) 2020, [orhun](mailto:orhunparmaksiz@gmail.com)