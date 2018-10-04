# read molecular stream from stdin, redirect it to geom_end.gen
cat > dftb_in.hsd

ln -s /share/apps/dftb+/SKFiles .
/share/apps/dftb+/bin/dftb+ > dftb.out

# output results
echo @model_properties_format_version 0.1
# output xyz coordinates
echo @structure
cat geo_end.xyz|tail -n +3
echo @energy
grep "Total energy:" detailed.out|awk '{print $3}'
