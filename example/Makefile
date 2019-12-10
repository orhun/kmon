PWD=$(shell pwd)
VER=$(shell uname -r)
KERNEL_BUILD=/lib/modules/$(VER)/build
obj-m += lkm_example.o

all:
	make -C $(KERNEL_BUILD) M=$(PWD) modules
install:
	make -C $(KERNEL_BUILD) M=$(PWD) modules_install
clean:
	make -C $(KERNEL_BUILD) M=$(PWD) clean
