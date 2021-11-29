KERNEL = julios
ISO = julios.iso
INSTALL_ROOT = iso
ABS_INSTALL = $(abspath $(INSTALL_ROOT))

LINKER_SCRIPT = src/linker.ld
BOOT_OBJS = src/multiboot.o src/boot.o
LIB_JULIOS = target/x86_64-julios/debug/libjulios.a

GRUB_CFG = grub/grub.cfg

SRC = src/lib.rs

all: $(ISO)

$(ISO): install
	./tools/create-iso.sh $@ $(INSTALL_ROOT)

install: $(KERNEL) $(GRUB_CFG)
	mkdir -p $(ABS_INSTALL)
	mkdir -p $(ABS_INSTALL)/boot/grub
	cp $(KERNEL) $(ABS_INSTALL)/boot
	cp grub/grub.cfg $(ABS_INSTALL)/boot/grub

$(KERNEL): $(LIB_JULIOS) $(LINKER_SCRIPT) $(BOOT_OBJS)
	ld -n -T $(LINKER_SCRIPT) -o $(KERNEL) $(BOOT_OBJS) $(LIB_JULIOS)

$(LIB_JULIOS): $(SRC)
	cargo build


%.o: %.asm
	nasm -f elf64 $^ -o $@

clean:
	$(RM) $(BOOT_OBJS)
	$(RM) $(KERNEL)
	$(RM) julios.iso
	$(RM) -r iso

.PHONY: $(INSTALL_ROOT) install clean all
