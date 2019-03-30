#! /usr/bin/env bash

## Create POSCAR from stdin with user defined template
cat > POSCAR

## Prepare other input files
# copy important files into the .tmp* scratch directory,
# which will be automatically removed if job finished.
cp ../vasp-input/* .

# PLEASE CHANGE
# submit vasp.
mpirun -np 10 ~/bin/vasp > run.log

# PLEASE CHANGE
# extract energy and forces from OUTCAR
~/rxe-vasp/bin/vasp-adaptor OUTCAR

