#!/bin/sh
docker run -e CC_aarch64_none_elf=aarch64-none-elf-gcc -e AR_aarch64_none_elf=aarch64-none-elf-ar -e 'CFLAGS_aarch64_none_elf=-L/opt/devkitpro/portlibs/switch/lib -L/opt/devkitpro/libnx/lib -L/opt/devkitpro/devkitA64/lib/gcc/aarch64-none-elf/8.2.0/pic -L/opt/devkitpro/devkitA64/aarch64-none-elf/lib/pic -Ltarget/aarch64-none-elf/release --sysroot=/opt/devkitpro/devkitA64/aarch64-none-elf/' -e CC=gcc -e AR=ar -e 'BINDGEN_LIBNX=1' -v "$HOME/.vimrc:/root/.vimrc" -v "$HOME/.vim:/root/.vim" --rm -it --net=host -v "$(pwd):/workdir" rusted-switch:latest /bin/bash
