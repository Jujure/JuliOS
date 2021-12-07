KERNEL = julios
ISO = julios.iso
INSTALL_ROOT = iso
ABS_INSTALL = $(abspath $(INSTALL_ROOT))

LINKER_SCRIPT = src/linker.ld
BOOT_OBJS = src/boot/multiboot.o src/boot/boot.o
LIB_JULIOS = target/x86_64-julios/debug/libjulios.a

GRUB_CFG = grub/grub.cfg

all: $(ISO)

run: $(ISO)
	qemu-system-x86_64 -cdrom $< -serial stdio -m 8G

debug: $(ISO)
	bochs -q

$(ISO): install
	grub-mkrescue -o $@ $(INSTALL_ROOT)

install: $(KERNEL) $(GRUB_CFG)
	mkdir -p $(ABS_INSTALL)
	mkdir -p $(ABS_INSTALL)/boot/grub
	cp $(KERNEL) $(ABS_INSTALL)/boot
	cp grub/grub.cfg $(ABS_INSTALL)/boot/grub

$(KERNEL): $(LIB_JULIOS) $(LINKER_SCRIPT) $(BOOT_OBJS)
	ld -n -T $(LINKER_SCRIPT) -o $(KERNEL) $(BOOT_OBJS) $(LIB_JULIOS)

$(LIB_JULIOS):
	cargo build


%.o: %.asm
	nasm -f elf64 $^ -o $@

clean:
	$(RM) $(BOOT_OBJS)
	$(RM) $(KERNEL)
	$(RM) julios.iso
	$(RM) -r iso
	$(RM) -r target

.PHONY: $(INSTALL_ROOT) install clean all run debug $(LIB_JULIOS)
