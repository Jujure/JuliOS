#!/bin/sh

iso_filename=$1
base_dir=$2

unset MFLAGS MAKEFLAGS

grub-mkrescue -o $iso_filename $base_dir
