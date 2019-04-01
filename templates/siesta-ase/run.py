import json
import ase.io

def set_calculator_for_sp(atoms):
    """single point calculation using dftb calculator

    Parameters
    ----------
    atoms: the ase Atoms object to be set
    """
    import os

    os.environ["SIESTA_COMMAND"] = "/share/apps/siesta/bin/siesta-bzr-serial < ./%s > ./%s"
    os.environ["SIESTA_PP_PATH"] = "/share/apps/siesta/pp"

    from ase.calculators.siesta import Siesta

    atoms.set_calculator(Siesta(
        xc="GGA",
        basis_set='DZP',
        kpts=[1, 1, 1],
    ))

def format_atoms_for_gosh_model(atoms):
    e = atoms.get_total_energy()
    f = atoms.get_forces()

    print("@model_properties_format_version 0.1\n")
    print("@structure")
    for s, p in zip(atoms.symbols, atoms.positions):
        line = "{:4}".format(s)
        line += "".join(["{:-20.12E}".format(v) for v in p])
        print(line)

    print("\n@energy")
    print("{:10.8}".format(e))
    print("\n@forces")
    for x in f:
        line = "".join(["{:-20.12E}".format(v) for v in x])
        print(line)

def main():
    atoms = ase.io.read("input.cif")
    set_calculator_for_sp(atoms)
    format_atoms_for_gosh_model(atoms)


if __name__ == '__main__':
    main()
