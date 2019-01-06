CROSS=aarch64-none-elf
CC=$(CROSS)-gcc
CFLAGS=-Wall -O -ffreestanding

BLINKY_OBJS=crt0.o blinky.o

all: blinky.bin

%.o: %.S
	$(CROSS)-gcc -c $< -o $@

%.o: %.c
	$(CROSS)-gcc -c $< -o $@

blinky.bin: blinky.elf
	$(CROSS)-objcopy -O binary $< $@

blinky.elf: $(BLINKY_OBJS) layout.ld
	$(CROSS)-ld $(BLINKY_OBJS) -o $@ -Tlayout.ld

clean:
	rm -f $(BLINKY_OBJS) *.bin *.elf
