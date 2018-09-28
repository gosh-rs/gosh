#! /usr/bin/env python
# imports

# [[file:~/Workspace/Programming/gosh/gosh.note::*imports][imports:1]]
import ase.io

from ase.neb import NEB
# imports:1 ends here

# balckbox model/calculator
# Universal External Model Interface

# [[file:~/Workspace/Programming/gosh/gosh.note::*balckbox%20model/calculator][balckbox model/calculator:1]]
import json
import subprocess
import re
from collections import defaultdict

from ase.calculators.calculator import FileIOCalculator, Calculator

class BBMCalculator(FileIOCalculator):
    implemented_properties = ['energy', 'forces']
    command = 'runner PREFIX.xyz -t /share/apps/mopac/sp'

    def __init__(self, restart=None, ignore_bad_restart_file=False,
                 label='bbm', atoms=None, **kwargs):
        """
        general external model caller interface
        """
        FileIOCalculator.__init__(self, restart, ignore_bad_restart_file,
                                  label, atoms, **kwargs)
        self.ncalls = 0

    def calculate(self, atoms=None, properties=['energy'], system_changes=["positions"]):
        Calculator.calculate(self, atoms, properties, system_changes)
        if self.command is None:
            raise CalculatorSetupError(
                'Please set ${} environment variable '
                .format('ASE_' + self.name.upper() + '_COMMAND') +
                'or supply the command keyword')

        ase.io.write("{}.xyz".format(self.prefix), self.atoms)
        command = self.command.replace('PREFIX', self.prefix)
        process = subprocess.Popen(command, stdout=subprocess.PIPE, shell=True, universal_newlines=True)
        output, errorcode = process.communicate()

        if errorcode:
            raise CalculationFailed('{} in {} returned an error: {}'
                                    .format(self.name, self.directory,
                                            errorcode))

        entries = parse_model_properties(output)
        self.results = entries[-1]
        self.ncalls += 1

def ase_results_to_json(calculator):
    """convert ase calculator results to json"""
    d = {}
    for k, v in calculator.results.items():
        # convert numpy array to plain list
        if k in ("forces", "dipole", "stress", "charges", "magmoms"):
            d[k] = v.tolist()
        else:
            d[k] = v
    return json.dumps(d)
# balckbox model/calculator:1 ends here



# parse model results

# [[file:~/Workspace/Programming/gosh/gosh.note::*balckbox%20model/calculator][balckbox model/calculator:2]]
def parse_one_part(part):
    if not part.strip():
        return

    dict_properties = defaultdict(list)
    k = None
    for line in part.strip().splitlines():
        if line.startswith("@"):
            k = line.strip()[1:]
        else:
            if k and not line.startswith("#"):
                dict_properties[k].append(line)

    return dict_properties

def refine_entry(entry):
    d = {}
    for k, v in entry.items():
        if k == "energy":
            d["energy"] = float(v[0])
        elif k == "forces":
            d["forces"] = []
            for line in v:
                x, y, z = [float(x) for x in line.split()]
                d["forces"].append([x, y, z])
        elif k == "dipole":
            d["dipole"] = np.array([float(x) for x in v[0].split()])

    d["forces"] = np.array(d["forces"])

    return d

def parse_model_properties(stream):
    """parse calculated properties"""

    parts = re.compile('^@model_properties_.*$', re.M).split(stream)
    all_entries = []
    for part in parts:
        entry = parse_one_part(part)
        if entry:
            d = refine_entry(entry)
            all_entries.append(d)

    return all_entries
# balckbox model/calculator:2 ends here

# mopac

# [[file:~/Workspace/Programming/gosh/gosh.note::*mopac][mopac:1]]
def set_mopac_calculator_for_sp(atoms):
    from ase.calculators.mopac import MOPAC

    # the default relscf parameter in ase is unnecessarily high
    # calc = MOPAC(method="PM6", label="scr/mopac", relscf=0.001)
    calc = BBMCalculator(label="scr/mopac")
    atoms.set_calculator(calc)
# mopac:1 ends here

# neb

# [[file:~/Workspace/Programming/gosh/gosh.note::*neb][neb:1]]
def run_neb(images, fmax=0.1, maxstep=100, cineb=True, ks=1):
    """run Nudged Elastic Band (NEB) calculation

    Parameters
    ----------
    images   : a list of ase atoms object as initial guess for NEB calculation
    cineb    : enable climbing image NEB or not
    keep_image_distance: adjust spring constant k to keep original image distance
    """
    from ase.optimize import BFGS, FIRE, LBFGS

    # set spring constants
    neb = NEB(images, remove_rotation_and_translation=True, climb=cineb, k=ks)
    n = BFGS(neb)
    n.run(fmax=fmax, steps=maxstep)

    return neb

def read_images(filename):
    """read images (multiple molecules) from filename"""
    images = ase.io.read(filename, index=":")
    return images

def get_neb_fmax(neb):
    """Returns fmax, as used by optimizers with NEB."""
    forces = neb.get_forces()
    return np.sqrt((forces**2).sum(axis=1).max())
# neb:1 ends here

# smart

# [[file:~/Workspace/Programming/gosh/gosh.note::*smart][smart:1]]
import os
import numpy as np
import subprocess
from ase.constraints import FixAtoms

def get_k_vars(images):
    n = len(images)
    ks = []
    print("k vars:")
    for i in range(n-1):
        d = images[i+1].positions - images[i].positions
        k = 1 / np.linalg.norm(d)
        ks.append(k)
        print("{:02}--{:02} = {:4.2f}".format(i, i+1, k))
    return ks

def smart_neb(path, keep_image_distance=False):
    """NEB optimization in a smarter way

    Parameters
    ----------
    path: path to file containing images
    """
    if path.find("boc") >= 0:
        scheme = "boc"
    elif path.find("lst") >= 0:
        scheme = "lst"
    elif path.find("idpp") >= 0:
        scheme = "idpp"
    else:
        raise RuntimeError("wrong scheme")
    print("interpolating using {} scheme".format(scheme))

    images = read_images(path)
    nimages = len(images)

    # 1. collect energies and find the middle image with the highest energy
    for image in images:
        set_mopac_calculator_for_sp(image)
        e = image.get_total_energy()
    imax = locate_ts_image_index(images)

    # 2. refine images and update energy
    middle = images[imax]

    new_images = interpolate_images(images[0], middle, images[-1], nimages=nimages, scheme=scheme)
    for i in range(1, nimages-1):
        # ignore middle structure
        images[i].set_positions(new_images[i].get_positions())
    ase.io.write("stage0.xyz", images, plain=True)
    if keep_image_distance:
        ks = get_k_vars(images)
    else:
        ks = 1

    # 3. stepwise NEB opt
    maxcycle=5
    fmax = 0.5
    imax = locate_ts_image_index(images)
    for k in range(maxcycle):
        print("cycle {}".format(k).center(80, "="))
        for i in range(1, imax):
            j = nimages - i -1
            optimize_images_only(images, [i, j], cineb=False, fmax=fmax, maxstep=10, ks=ks)
            # locate_ts_image_index(images)

        # imax = locate_ts_image_index(images)
        # opt the quasi-ts image
        optimize_images_only(images, [imax], cineb=True, fmax=fmax, maxstep=5, ks=ks)
        locate_ts_image_index(images)

        unconstrain_images(images)
        neb = run_neb(images, fmax=fmax, cineb=True, maxstep=1, ks=ks)
        if get_neb_fmax(neb) < fmax:
            print("done")
            break

    ase.io.write("neb-images.xyz", images)
    imax = locate_ts_image_index(images)
    ase.io.write("ts.xyz", images[imax], plain=True)
    ncs = [image._calc.ncalls for image in images]
    print("total number of energy calls: {}".format(sum(ncs)))
    print(ncs)

def optimize_images_only(images, indices, cineb=False, maxstep=10, fmax=0.5, ks=1):
    """NEB opt images with indices while fixing the other images"""
    s = ",".join([str(i) for i in indices])
    print("opt images: {}".format(s))

    constrain_images(images)
    for i in indices:
        unconstrain_image(images[i])
    run_neb(images, fmax=fmax, maxstep=maxstep, cineb=cineb, ks=ks)

def get_highest_energy_index(energies):
    """Find the index of the image with the highest energy."""
    valid_entries = [(i, e) for i, e in enumerate(energies) if e == e]
    highest_energy_index = max(valid_entries, key=lambda x: x[1])[0]
    return highest_energy_index

def interpolate_images(reactant, middle, product, nimages=11, scheme="boc"):
    """
    create a new images connecting reactant, middle and product molecules
    """
    try:
        os.makedirs("scr")
    except FileExistsError:
        pass
    reactant.write("scr/r.xyz", plain=True)
    product.write("scr/p.xyz", plain=True)
    middle.write("scr/m.xyz", plain=True)

    if scheme == "boc":
        cmdline = "rxview scr/r.xyz scr/p.xyz -m scr/m.xyz scr/images.xyz -n {} -b".format(nimages)
        subprocess.call(cmdline.split())
        images = read_images("scr/images.xyz")
    elif scheme =="lst":
        cmdline = "rxview scr/r.xyz scr/p.xyz -m scr/m.xyz scr/images.xyz --single -n {}".format(nimages)
        subprocess.call(cmdline.split())
        images = read_images("scr/images.xyz")
    elif scheme == "idpp":
        images = create_idpp_images_with_middle(reactant, middle, product, nimages)

    return images

def create_idpp_images(initial, final, nimages):
    # create nimages
    images = [initial]
    images += [initial.copy() for i in range(nimages-2)]
    images += [final]
    neb = NEB(images, remove_rotation_and_translation=True, method="improvedtangent")

    # run linear or IDPP interpolation
    neb.interpolate("idpp")

    return images

def create_idpp_images_with_middle(reactant, middle, product, nimages):
    npart = nimages // 2
    rem   = nimages % 2
    images_left = create_idpp_images(reactant, middle, nimages=npart+rem)
    images_right = create_idpp_images(middle, product, nimages=npart+1)

    images = images_left[:]
    images.extend(images_right[1:])
    assert len(images) == nimages
    return images

def locate_ts_image_index(images):
    energies = []
    for i, image in enumerate(images):
        e = image.get_total_energy()
        energies.append(e)
    print(energies)
    imax = get_highest_energy_index(energies)
    return imax

def constrain_image(image):
    c = FixAtoms(mask=[True for i in range(len(image))])
    image.set_constraint(c)

def constrain_images(images):
    for image in images:
        constrain_image(image)

def unconstrain_image(image):
    image.constraints = []

def unconstrain_images(images):
    for image in images:
        unconstrain_image(image)
# smart:1 ends here

# test

# [[file:~/Workspace/Programming/gosh/gosh.note::*test][test:1]]
if __name__ == '__main__':
    import sys
    path = sys.argv[1]
    smart_neb(path)
# test:1 ends here
