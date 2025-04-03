#! /usr/bin/env bash

## 0. load environment variables
source ~/apps/env.rc >/dev/null
apps load orca >/dev/null

cp $BBM_TPL_DIR/orca.inp .
cat > orca-input.xyz
`which orca` orca.inp 2> $BBM_JOB_DIR/orca.err |tee $BBM_JOB_DIR/orca.log >orca.out

# copy orca out files back
#cp ./* "$BBM_JOB_DIR/"

# parse enregy and forces
echo @model_properties_format_version 0.1
natoms=$(head -n 1 orca-input.xyz | awk '{print $1}')

gosh-parser -- <<EOF
load orca.engrad
select-lines 8
println "@energy unit_factor=27.211386024367243"
print-selection
println "@forces  unit_factor=-51.422067090480645"
goto-line 12
select-lines --relative 1-$((natoms*3+1))
print-selection --pipe "xargs -n 3"
EOF

