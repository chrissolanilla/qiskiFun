import os
import matplotlib.pyplot as plt

from sabre.circuits import random_cx_circuit
from sabre.compile import compile_with_sabre
from sabre.targets import line_topology


def main():
    os.makedirs("data/images", exist_ok=True)

    num_qubits = 8
    circuit_depth = 40
    circuit_seed = 0
    transpile_seed = 0

    qc = random_cx_circuit(
        num_qubits=num_qubits,
        depth=circuit_depth,
        seed=circuit_seed,
    )

    compiled = compile_with_sabre(
        qc,
        coupling_map=line_topology(num_qubits),
        seed=transpile_seed,
    )

    # save original by itself
    fig1 = plt.figure(figsize=(16, 6))
    ax1 = fig1.add_subplot(111)
    qc.draw("mpl", ax=ax1)
    ax1.set_title(f"original circuit (seed={circuit_seed})")
    fig1.tight_layout()
    fig1.savefig("data/images/original.png", dpi=300, bbox_inches="tight")
    fig1.savefig("data/images/original.svg", bbox_inches="tight")
    plt.close(fig1)

    # save compiled by itself
    fig2 = plt.figure(figsize=(20, 8))
    ax2 = fig2.add_subplot(111)
    compiled.draw("mpl", ax=ax2)
    ax2.set_title(f"compiled with sabre (transpile_seed={transpile_seed})")
    fig2.tight_layout()
    fig2.savefig("data/images/compiled.png", dpi=300, bbox_inches="tight")
    fig2.savefig("data/images/compiled.svg", bbox_inches="tight")
    plt.close(fig2)

    # save side-by-side comparison
    fig3, axs = plt.subplots(2, 1, figsize=(20, 12))
    qc.draw("mpl", ax=axs[0])
    axs[0].set_title("original circuit")

    compiled.draw("mpl", ax=axs[1])
    axs[1].set_title("compiled circuit (sabre)")

    fig3.tight_layout()
    fig3.savefig("data/images/compare_vertical.png", dpi=300, bbox_inches="tight")
    fig3.savefig("data/images/compare_vertical.svg", bbox_inches="tight")
    plt.close(fig3)

    print("saved:")
    print("  data/images/original.png")
    print("  data/images/original.svg")
    print("  data/images/compiled.png")
    print("  data/images/compiled.svg")
    print("  data/images/compare_vertical.png")
    print("  data/images/compare_vertical.svg")


if __name__ == "__main__":
    main()
