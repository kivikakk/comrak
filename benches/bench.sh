#! /bin/bash

PROG=$1
ROOTDIR=$(git rev-parse --show-toplevel)

for lang in ar az be ca cs de en eo es es-ni fa fi fr hi hu id it ja ko mk nl no-nb pl pt-br ro ru sr th tr uk vi zh zh-tw; do \
    cat $ROOTDIR/vendor/progit/$lang/*/*.markdown | $PROG > /dev/null
done