#! /usr/bin/env python3
# imports

# [[file:~/Workspace/Programming/gosh/gosh.note::*imports][imports:1]]
import os

import ase
import ase.io
import numpy as np
import subprocess as sp

from ase.neb import NEB
# imports:1 ends here



# - 参数文件放在./SKFiles目录里.
# - dftb+要在命令搜索路径里(PATH).
# - 修改: Hamiltonian_MaxAngularMomentum_*


# [[file:~/Workspace/Programming/gosh/gosh.note::*dftb+][dftb+:2]]
def set_dftb_calculator_for_opt(atoms):
    """optimize the structure using dftb calculator in ase

    Parameters
    ----------
    atoms: the ase Atoms object to be set
    """
    import os
    os.environ["DFTB_COMMAND"] = "dftb+"
    os.environ["DFTB_PREFIX"] = "/home/ybyygu/Incoming/liuxc-dftb+/dftb-params/3ob-3-1/"

    from ase.calculators.dftb import Dftb

    atoms.set_calculator(Dftb(run_manyDftb_steps=True,
                              Driver_='ConjugateGradient',
                              Driver_MaxForceComponent='1E-3',
                              Driver_MaxSteps=50,
                              atoms=atoms,
                              Hamiltonian_MaxAngularMomentum_C='"p"',
                              Hamiltonian_MaxAngularMomentum_O='"p"',
                              Hamiltonian_MaxAngularMomentum_N='"p"',
                              Hamiltonian_MaxAngularMomentum_H='"s"',
    ))

def set_dftb_calculator_for_sp(atoms):
    """single point calculation using dftb calculator

    Parameters
    ----------
    atoms: the ase Atoms object to be set
    """
    import os
    os.environ["DFTB_COMMAND"] = "dftb+"
    os.environ["DFTB_PREFIX"] = "/home/ybyygu/Incoming/liuxc-dftb+/dftb-params/3ob-3-1"

    from ase.calculators.dftb import Dftb

    atoms.set_calculator(Dftb(atoms=atoms,
                              Hamiltonian_MaxAngularMomentum_C='"p"',
                              Hamiltonian_MaxAngularMomentum_O='"p"',
                              Hamiltonian_MaxAngularMomentum_N='"p"',
                              Hamiltonian_MaxAngularMomentum_H='"s"',
    ))


def dftb_opt(filename):
    """read atoms from filename, and optimize using dftb+, then save back inplace."""

    atoms = ase.io.read(filename)
    set_dftb_calculator_for_opt(atoms)
    e = atoms.get_total_energy()
    print("opt energy = {:-10.4f}".format(e))
    # avoid the bug for the xyz comment line
    atoms.write(filename, plain=True)
    print("updated structure inplace: {}".format(filename))
# dftb+:2 ends here

# gaussian

# [[file:~/Workspace/Programming/gosh/gosh.note::*gaussian][gaussian:1]]
def set_gaussian_calculator(atoms):
    from ase.calculators.gaussian import Gaussian

    calc = Gaussian(method="b3lyp",
                    basis="6-31g**",
                    nproc=4)
    atoms.set_calculator(calc)
# gaussian:1 ends here

# GEMI calculator
# General External Model Interface

# [[file:~/Workspace/Programming/gosh/gosh.note::*GEMI%20calculator][GEMI calculator:1]]
import json
import subprocess
import re
from collections import defaultdict

from ase.calculators.calculator import FileIOCalculator, Calculator

class GEMI(FileIOCalculator):
    implemented_properties = ['energy', 'forces']
    command = 'runner PREFIX.xyz -t /share/apps/mopac/sp'

    def __init__(self, restart=None, ignore_bad_restart_file=False,
                 label='gemi', atoms=None, **kwargs):
        """
        general external model caller interface
        """
        FileIOCalculator.__init__(self, restart, ignore_bad_restart_file,
                                  label, atoms, **kwargs)

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
# GEMI calculator:1 ends here



# tests

# [[file:~/Workspace/Programming/gosh/gosh.note::*GEMI%20calculator][GEMI calculator:2]]
def test_gemi():
    from ase.optimize import BFGS

    atoms = ase.io.read("./final.xyz")
    calc = GEMI()
    atoms.set_calculator(calc)
    n = BFGS(atoms)
    n.run(fmax=0.1)
# GEMI calculator:2 ends here



# parse model results

# [[file:~/Workspace/Programming/gosh/gosh.note::*GEMI%20calculator][GEMI calculator:3]]
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
# GEMI calculator:3 ends here

# mopac

# [[file:~/Workspace/Programming/gosh/gosh.note::*mopac][mopac:1]]
def set_mopac_calculator_for_sp(atoms):
    from ase.calculators.mopac import MOPAC

    # the default relscf parameter in ase is unnecessarily high
    calc = MOPAC(method="PM6", relscf=0.001)
    atoms.set_calculator(calc)

def set_mopac_calculator_for_opt(atoms):
    from ase.calculators.mopac import MOPAC

    # the default relscf parameter in ase is unnecessarily high
    calc = MOPAC(method="PM6", task='GRADIENTS', relscf=0.001)
    atoms.set_calculator(calc)

def mopac_opt(filename):
    """read atoms from filename, and optimize using dftb+, then save back inplace."""

    atoms = ase.io.read(filename)
    set_mopac_calculator_for_opt(atoms)
    e = atoms.get_total_energy()
    print("opt energy = {:-10.4f}".format(e))
    # avoid the bug for the xyz comment line
    atoms.write(filename, plain=True)
    print("updated structure inplace: {}".format(filename))
# mopac:1 ends here

# dmol3

# [[file:~/Workspace/Programming/gosh/gosh.note::*dmol3][dmol3:1]]
def set_dmol3_calculator(atoms):
    from ase.calculators.dmol import DMol3

    dmol3 = DMol3(symmetry='off',
                  spin_polarization='restricted',
                  charge=0,
                  functional='blyp',
                  basis='dnd',
                  scf_iterations='-100',
                  initial_hessian='improved',
                  pseudopotential='none',
                  integration_grid='medium',
                  aux_density='octupole',
                  occupation='fermi',
                  scf_charge_mixing=0.2,
                  scf_diis='6 pulay',
                  scf_density_convergence=1.0e-5)
    atoms.set_calculator(dmol3)
# dmol3:1 ends here

# batch neb
# calculate NEB images in batch to reduce IO costs


# [[file:~/Workspace/Programming/gosh/gosh.note::*batch%20neb][batch neb:1]]
class BatchNEB(NEB):
    def __init__(self, images, remove_rotation_and_translation=False, **kwargs):
        NEB.__init__(self, images, remove_rotation_and_translation=False, **kwargs)

    def get_forces(self):
        import subprocess

        e0 = self.images[0].get_total_energy()
        e1 = self.images[-1].get_total_energy()

        ase.io.write("neb.xyz", self.images[1:-1])
        cmdline = "runner neb.xyz -j -t /share/apps/mopac/sp"
        process = subprocess.Popen(cmdline, stdout=subprocess.PIPE, shell=True, universal_newlines=True)
        output, err = process.communicate()
        if err:
            raise RuntimeError("runner failed!")

        entries = parse_model_properties(output)
        n = len(self.images)
        assert n - 2 == len(entries)

        for i in range(1, n-1):
            image = self.images[i]
            # ase is too complex than needed
            image._calc.atoms = image.copy()
            image._calc.results = entries[i-1]

        return NEB.get_forces(self)
# batch neb:1 ends here



# test

# [[file:~/Workspace/Programming/gosh/gosh.note::*batch%20neb][batch neb:2]]
def test_batch_neb(path):
    import time

    from ase.optimize import BFGS

    def get_images(neb=False):
        images= ase.io.read(path, index=":")
        for image in images:
            if not neb:
                calc = GEMI()
                image.set_calculator(calc)
            else:
                set_mopac_calculator_for_sp(image)
        return images

    print("start batched NEB...", time.ctime())
    neb = BatchNEB(get_images())
    n = BFGS(neb)
    n.run(fmax=0.2, steps=10)
    print("done, {}", time.ctime())

    # external NEB
    print("start external NEB...", time.ctime())
    neb = NEB(get_images())
    n = BFGS(neb)
    n.run(fmax=0.2, steps=10)
    print("done, {}", time.ctime())

    # normal NEB
    print("start normal NEB...", time.ctime())
    neb = NEB(get_images(neb=True))
    n = BFGS(neb)
    n.run(fmax=0.2, steps=10)
    print("done, {}", time.ctime())
# batch neb:2 ends here

# neb

# [[file:~/Workspace/Programming/gosh/gosh.note::*neb][neb:1]]
def create_neb_images(reactantfile,
                      productfile,
                      nimages=11,
                      outfilename=None,
                      format=None,
                      scheme='idpp'):
    """
    interpolate images from reactant to product for NEB calculation

    Parameters
    ----------
    reactantfile, productfile : filename containing reactant/product molecule
    outfilename               : save images as outfilename
    format                    : set outfile format
    nimages                   : the number of images
    scheme                    : linear or idpp scheme for interpolation
    """
    # read initial and final states:
    initial = ase.io.read(reactantfile, format=format)
    final = ase.io.read(productfile, format=format)

    # create nimages
    images = [initial]
    images += [initial.copy() for i in range(nimages-2)]
    images += [final]
    neb = NEB(images, remove_rotation_and_translation=True, method="improvedtangent")

    # run linear or IDPP interpolation
    neb.interpolate(scheme)

    # calculate image distances
    for i in range(len(images)-1):
        image_this = images[i]
        image_next = images[i+1]
        diff = image_next.positions - image_this.positions
        distance = np.linalg.norm(diff)
        print("diff = {:6.3f}".format(distance))

    if outfilename:
        ase.io.write(outfilename, images)

    return images

def run_neb(images, trajfile, fmax=0.1, maxstep=100, cineb=True, keep_image_distance=True, batch=False):
    """run Nudged Elastic Band (NEB) calculation

    Parameters
    ----------
    images   : a list of ase atoms object as initial guess for NEB calculation
    trajfile : trajectory file name during optimization
    cineb    : enable climbing image NEB or not
    keep_image_distance: adjust spring constant k to keep original image distance
    """
    from ase.optimize import BFGS, FIRE, LBFGS

    imax = locate_ts_image_index(images)

    # set spring constants
    if keep_image_distance:
        n = len(images)
        ks = []
        print("k vars:")
        for i in range(n-1):
            d = images[i+1].positions - images[i].positions
            k = 1 / np.linalg.norm(d)
            ks.append(k)
            print("{:02}--{:02} = {:4.2f}".format(i, i+1, k))
    else:
        ks = 1

    if batch:
        neb = BatchNEB(images, remove_rotation_and_translation=False, climb=cineb, k=ks)
    else:
        neb = NEB(images, remove_rotation_and_translation=True, climb=cineb, k=ks)

    # n = FIRE(neb, trajectory=trajfile, force_consistent=False)
    # n = FIRE(neb, trajectory=trajfile)
    # n = LBFGS(neb, trajectory=trajfile, force_consistent=False)
    n = BFGS(neb, trajectory=trajfile)
    n.run(fmax=fmax, steps=maxstep)

    return neb

def read_images(filename):
    """read images (multiple molecules) from filename"""
    images = ase.io.read(filename, index=":")
    return images

def locate_ts_image_index(images):
    energies = []
    for i, image in enumerate(images):
        e = image.get_total_energy()
        energies.append(e)
    print(energies)
    imax = get_highest_energy_index(energies)
    return imax

def get_highest_energy_index(energies):
    """Find the index of the image with the highest energy."""
    valid_entries = [(i, e) for i, e in enumerate(energies) if e == e]
    highest_energy_index = max(valid_entries, key=lambda x: x[1])[0]
    return highest_energy_index
# neb:1 ends here

# ts

# [[file:~/Workspace/Programming/gosh/gosh.note::*ts][ts:1]]
def ts_search(images_filename, method, label=None, maxstep=20, keep_image_distance=True, climbing=False):
    """the main entry point for transition state searching

    images_filename: the filename containing multiple molecules (images)
    maxstep        : the max allowed number of steps
    """
    # load data
    images = read_images(images_filename)
    print('loaded {} images'.format(len(images)))

    batch = False
    for image in images:
        # using dftb+
        if method == "dftb":
            set_dftb_calculator_for_sp(image)
        elif method == "gaussian":
            set_gaussian_calculator(image)
        elif method == "mopac":
            set_mopac_calculator_for_sp(image)
        else:
            # using external model calculator
            calc = GEMI()
            image.set_calculator(calc)
            batch = True

    if batch:
        print("*enable batch calculations...*")

    # create working dirs
    if label is None:
        label, _ = os.path.splitext(os.path.basename(images_filename))
    print("created working directory: {}".format(label))
    os.makedirs(label, exist_ok=True)
    os.chdir(label)

    # start neb calculation without climbing image
    if not climbing:
        trajfile = '{}.traj'.format(label)
        neb = run_neb(images, trajfile,
                      maxstep=maxstep,
                      fmax=0.5,
                      cineb=False,
                      keep_image_distance=keep_image_distance,
                      batch=batch)
    else:
        # climbing
        print("climbing...")
        trajfile = '{}-ci.traj'.format(label)
        neb = run_neb(images, trajfile,
                      maxstep=maxstep,
                      fmax=0.1,
                      cineb=True,
                      keep_image_distance=keep_image_distance,
                      batch=batch)

    # a brief summary
    for i, image in enumerate(images):
        energy = image.get_potential_energy()
        print("image {:02}: energy = {:<-12.4f} eV".format(i, energy))

    # find ts
    tmp = [(image.get_total_energy(), image) for image in neb.images]
    tmp.sort(key=lambda pair: pair[0], reverse=True)
    _, ts = tmp[0]
    ts.write("ts.xyz")

    # write optimized images
    if not climbing:
        ase.io.write("neb-images.pdb", neb.images)
    else:
        ase.io.write("cineb-images.pdb", neb.images)

    # goto workdir
    os.chdir("..")

    return neb
# ts:1 ends here

# batch

# [[file:~/Workspace/Programming/gosh/gosh.note::*batch][batch:1]]
def run_boc(nimages, method, keep=True):
    cmdline = "rxview reactant.xyz product.xyz rx-boc.xyz -n {} -b --gap".format(nimages)
    sp.run(cmdline.split())
    ts_search("rx-boc.xyz", maxstep=500, method=method, keep_image_distance=keep, climbing=False)
    ts_search("rx-boc/neb-images.pdb", label="rx-boc", maxstep=500, method=method, keep_image_distance=keep, climbing=True)

def run_lst(nimages, method, keep=False):
    if keep:
        print("set k vars to fit original image distances ...")
        cmdline = "rxview reactant.xyz product.xyz rx-lst.xyz -n {} --gap".format(nimages)
    else:
        print("set k = 1, to keep image evenly distributed ...")
        cmdline = "rxview reactant.xyz product.xyz rx-lst.xyz -n {} --single".format(nimages)

    sp.run(cmdline.split())
    ts_search("rx-lst.xyz", maxstep=500, method=method, keep_image_distance=keep, climbing=False)
    ts_search("rx-lst/neb-images.pdb", label="rx-lst", maxstep=500, method=method, keep_image_distance=keep, climbing=True)

def run_idpp(nimages, method, keep=False):
    # create rxview images
    create_neb_images("reactant.xyz", "product.xyz", outfilename="idpp.pdb", scheme="idpp", nimages=nimages)
    # for idpp using the normal way
    ts_search("idpp.pdb", maxstep=500, method=method, keep_image_distance=keep, climbing=False)
    ts_search("idpp/neb-images.pdb", label="idpp", maxstep=500, method=method, keep_image_distance=keep, climbing=True)

def run_all(method="dftb"):
    nimages = 11
    run_boc(nimages, method)
    run_lst(nimages, method)
    run_idpp(nimages, method)

if __name__ == '__main__':
    run_all("gemi")
# batch:1 ends here
