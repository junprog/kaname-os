.DEFAULT_GOAL := build

KERNEL := target/x86_64-unknown-none/debug/kaname-kernel
IMAGE := boot/kaname.img
LIMINE_VERSION := 12.9.4
LIMINE_DIR := boot/dist/limine
LIMINE_EFI := $(LIMINE_DIR)/BOOTX64.EFI
OVMF_CODE := /usr/share/OVMF/OVMF_CODE_4M.fd
OVMF_VARS_TEMPLATE := /usr/share/OVMF/OVMF_VARS_4M.fd
OVMF_VARS := boot/OVMF_VARS.fd

.PHONY: build image run clean check-deps

check-deps:
	@for cmd in cargo curl tar mkfs.vfat mmd mcopy qemu-system-x86_64; do \
		command -v $$cmd >/dev/null || { echo "Missing: $$cmd"; exit 1; }; \
	done
	@test -f $(OVMF_CODE) || { echo "Missing: $(OVMF_CODE)"; exit 1; }
	@test -f $(OVMF_VARS_TEMPLATE) || { echo "Missing: $(OVMF_VARS_TEMPLATE)"; exit 1; }

build:
	cargo rustc -p kaname-kernel -- \
		-C relocation-model=static \
		-C link-arg=-Tlinker.ld \
		-C link-arg=-zmax-page-size=0x1000

$(LIMINE_EFI):
	mkdir -p $(LIMINE_DIR)
	curl -fL --retry 3 \
		https://github.com/limine-bootloader/limine/releases/download/v$(LIMINE_VERSION)/limine-binary.tar.gz \
		-o boot/dist/limine-binary.tar.gz
	tar -xzf boot/dist/limine-binary.tar.gz \
		-C $(LIMINE_DIR) --strip-components=1
	test -f $(LIMINE_EFI)

image: check-deps build $(LIMINE_EFI)
	truncate -s 64M $(IMAGE)
	mkfs.vfat -F 32 $(IMAGE)
	mmd -i $(IMAGE) ::/EFI
	mmd -i $(IMAGE) ::/EFI/BOOT
	mmd -i $(IMAGE) ::/boot
	mcopy -i $(IMAGE) $(LIMINE_EFI) ::/EFI/BOOT/BOOTX64.EFI
	mcopy -i $(IMAGE) boot/image/EFI/BOOT/limine.conf ::/EFI/BOOT/limine.conf
	mcopy -i $(IMAGE) $(KERNEL) ::/boot/kaname-kernel

$(OVMF_VARS):
	cp $(OVMF_VARS_TEMPLATE) $(OVMF_VARS)

run: image $(OVMF_VARS)
	qemu-system-x86_64 \
		-machine q35 \
		-m 512M \
		-drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
		-drive if=pflash,format=raw,file=$(OVMF_VARS) \
		-drive format=raw,file=$(IMAGE),if=virtio \
		-boot order=c \
		-debugcon stdio \
		-global isa-debugcon.iobase=0xe9

clean:
	cargo clean
	rm -f $(IMAGE)
