#! /usr/bin/env bash
# submit.sh

# read molecular stream from stdin, redirect it to geom_end.gen
cat > mopac.mop
mopac mopac.mop 2> err.log

# call adaptor to extract results
mopac-adaptor mopac.out -a
