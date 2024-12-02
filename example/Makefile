PWD=$(shell pwd)
VER=$(shell uname -r)
KERNEL_BUILD=/lib/modules/$(VER)/build
obj-m += lkm_example.o

all:
	$(MAKE) -C $(KERNEL_BUILD) M=$(PWD) modules
install:
	$(MAKE) -C $(KERNEL_BUILD) M=$(PWD) modules_install
clean:
	$(MAKE) -C $(KERNEL_BUILD) M=$(PWD) clean
