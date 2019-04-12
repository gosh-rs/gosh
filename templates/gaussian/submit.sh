#! /usr/bin/env bash

# read input stream from stdin
g09 > gaussian.log 2> err.log

# parsing results from formatted checkpoint file
/share/apps/gaussian/bin/gaussian-adaptor Test.FChk
