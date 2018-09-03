#! /usr/bin/env python3
# [[file:~/Workspace/Programming/gosh/gosh.note::*imports][imports:1]]
import os

import ase
import ase.io
import numpy as np

from ase.neb import NEB
# imports:1 ends here

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

# [[file:~/Workspace/Programming/gosh/gosh.note::*gaussian][gaussian:1]]
def set_gaussian_calculator(atoms):
    from ase.calculators.gaussian import Gaussian

    calc = Gaussian(method="b3lyp",
                    basis="6-31g**",
                    nproc=4)
    atoms.set_calculator(calc)
# gaussian:1 ends here

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

def run_neb(images, trajfile, fmax=0.05, maxstep=100, cineb=True):
    """run Nudged Elastic Band (NEB) calculation

    Parameters
    ----------
    images   : a list of ase atoms object as initial guess for NEB calculation
    trajfile : trajectory file name during optimization
    cineb    : enable climbing image NEB or not
    """
    from ase.optimize import BFGS, FIRE

    neb = NEB(images, remove_rotation_and_translation=True, climb=cineb)
    # n = FIRE(neb, trajectory=trajfile, force_consistent=False)
    # n = FIRE(neb, trajectory=trajfile)
    n = BFGS(neb, trajectory=trajfile)
    n.run(fmax=fmax, steps=maxstep)
    return neb

def read_images(filename):
    """read images (multiple molecules) from filename"""
    images = ase.io.read(filename, index=":")
    return images
# neb:1 ends here

# [[file:~/Workspace/Programming/gosh/gosh.note::*ts][ts:1]]
def ts_search(images_filename, fmax=0.05, maxstep=20, cineb=False):
    """the main entry point for transition state searching

    images_filename: the filename containing multiple molecules (images)
    fmax           : the max force for convergence
    maxstep        : the max allowed number of steps
    cineb          : using climbing image or not in NEB searching
    """

    # load data
    images = read_images(images_filename)
    print('loaded {} images'.format(len(images)))

    label, _ = os.path.splitext(os.path.basename(images_filename))
    os.makedirs(label, exist_ok=True)
    os.chdir(label)
    print("created working directory: {}".format(label))

    # using dftb+
    for image in images:
        set_dftb_calculator_for_sp(image)

    # start neb calculation
    trajfile = '{}.traj'.format(label)
    neb = run_neb(images, trajfile, fmax=fmax, maxstep=maxstep, cineb=cineb)

    # a brief summary
    for i, image in enumerate(images):
        energy = image.get_potential_energy()
        print("image {:02}: energy = {:<-12.4f} eV".format(i, energy))

    # write optimized images
    ase.io.write("neb-images.xyz", neb.images)

    # goto workdir
    os.chdir("..")

    return neb
# ts:1 ends here

# [[file:~/Workspace/Programming/gosh/gosh.note::*batch][batch:1]]
def run_all():
    nimages = 11
    import subprocess as sp
    cmdline = "babel reactant.mol2 reactant.xyz"
    sp.run(cmdline.split())

    cmdline = "babel product.mol2 product.xyz"
    sp.run(cmdline.split())

    # pre-optimization
    dftb_opt("reactant.xyz")
    dftb_opt("product.xyz")

    # rxview images
    # LST style
    cmdline = "rxview reactant.mol2 product.mol2 rx-lst.xyz -n {}".format(nimages)
    sp.run(cmdline.split())

    # BOC
    cmdline = "rxview reactant.mol2 product.mol2 rx-boc.xyz -n {} -b".format(nimages)
    sp.run(cmdline.split())

    # idpp images
    create_neb_images("reactant.xyz", "product.xyz", outfilename="idpp.pdb", scheme="idpp", nimages=11)

    ts_search("rx-boc.xyz", maxstep=500, cineb=True, fmax=0.1)

    ts_search("rx-lst.xyz", maxstep=500, cineb=True, fmax=0.1)

    ts_search("idpp.pdb", maxstep=500, cineb=True, fmax=0.1)

if __name__ == '__main__':
    run_all()
# batch:1 ends here
