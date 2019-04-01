#! /usr/bin/env bash
# submit.sh

# prepare inputs
ln -s /share/apps/siesta/pp/* .
ln -s /share/apps/siesta/sp/default.fdf .

# reuse density file
#cp /tmp/siesta.DM . 2>/dev/null

# read fdf stream from stdin
mpirun -np 4 /share/apps/siesta/bin/siesta > siesta.log

# save DM file for caching
#cp -f siesta.DM /tmp/

/share/apps/siesta/bin/siesta-adaptor siesta.log
