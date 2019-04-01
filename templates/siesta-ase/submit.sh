#! /usr/bin/env bash
# submit.sh

cat > input.cif
python /share/apps/siesta/ase/run.py > run.log
