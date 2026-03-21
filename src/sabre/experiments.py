from .circuits import random_cx_circuit
from .compile import compile_with_sabre
from .metrics import circuit_metrics

def run_experiment(coupling_map, num_qubits=8, circuit_depth=30, circuit_seed=1, transpile_seed=11):
    original = random_cx_circuit(num_qubits=num_qubits, depth=circuit_depth, seed=circuit_seed)
    compiled = compile_with_sabre(
        original,
        coupling_map=coupling_map,
        optimization_level=3,
        seed=transpile_seed,
    )

    original_metrics = circuit_metrics(original)
    compiled_metrics = circuit_metrics(compiled)

    row = {
        "circuit_seed": circuit_seed,
        "transpile_seed": transpile_seed,
        **{f"orig_{k}": v for k, v in original_metrics.items()},
        **{f"comp_{k}": v for k, v in compiled_metrics.items()},
    }
    return original, compiled, row
