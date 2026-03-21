use num_complex::Complex64;
use std::collections::{HashMap, VecDeque};
use std::env;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Gate {
    H(usize),
    X(usize),
    CX(usize, usize),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum PhysicalGate {
    H(usize),
    X(usize),
    CX(usize, usize),
    SWAP(usize, usize),
}

#[derive(Clone, Debug)]
struct Circuit {
    num_qubits: usize,
    gates: Vec<Gate>,
}

#[derive(Clone, Debug)]
struct Hardware {
    num_qubits: usize,
    edges: Vec<(usize, usize)>,
}

#[derive(Clone, Debug)]
struct SearchState {
    // mapping[logical] = physical
    mapping: Vec<usize>,
    gate_index: usize,
    emitted: Vec<PhysicalGate>,
    cnot_cost: usize,
    swap_count: usize,
}

#[derive(Clone, Debug)]
struct Solution {
    emitted: Vec<PhysicalGate>,
    cnot_cost: usize,
    swap_count: usize,
}

fn parse_index(token: &str) -> Option<usize> {
    let start = token.find('[')?;
    let end = token.find(']')?;
    token[start + 1..end].parse().ok()
}

fn parse_qasm_file(path: &Path) -> Result<Circuit, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("failed to read file: {e}"))?;

    let mut num_qubits = 0usize;
    let mut gates = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        if line.starts_with("OPENQASM") || line.starts_with("include ") || line.starts_with("bit[") {
            continue;
        }

        if line.starts_with("qubit[") {
            let start = line.find('[').ok_or("bad qubit declaration")?;
            let end = line.find(']').ok_or("bad qubit declaration")?;
            num_qubits = line[start + 1..end]
                .parse()
                .map_err(|_| "bad qubit count".to_string())?;
            continue;
        }

        if line.starts_with("barrier") {
            continue;
        }

        if line.contains("measure") {
            // ignore measurements for unitary equivalence
            continue;
        }

        if line.starts_with("h ") {
            let q = parse_index(line).ok_or("bad h gate")?;
            gates.push(Gate::H(q));
            continue;
        }

        if line.starts_with("x ") {
            let q = parse_index(line).ok_or("bad x gate")?;
            gates.push(Gate::X(q));
            continue;
        }

        if line.starts_with("cx ") {
            let rest = line.trim_start_matches("cx ").trim_end_matches(';');
            let parts: Vec<_> = rest.split(',').map(|s| s.trim()).collect();
            if parts.len() != 2 {
                return Err(format!("bad cx line: {line}"));
            }
            let c = parse_index(parts[0]).ok_or("bad cx control")?;
            let t = parse_index(parts[1]).ok_or("bad cx target")?;
            gates.push(Gate::CX(c, t));
            continue;
        }

        return Err(format!("unsupported line: {line}"));
    }

    if num_qubits == 0 {
        return Err("did not find qubit declaration".to_string());
    }

    Ok(Circuit { num_qubits, gates })
}

fn line_hardware(n: usize) -> Hardware {
    let mut edges = Vec::new();
    for i in 0..n.saturating_sub(1) {
        edges.push((i, i + 1));
    }
    Hardware { num_qubits: n, edges }
}

fn has_edge(hw: &Hardware, a: usize, b: usize) -> bool {
    hw.edges
        .iter()
        .any(|&(u, v)| (u == a && v == b) || (u == b && v == a))
}

fn identity_mapping(n: usize) -> Vec<usize> {
    (0..n).collect()
}

fn apply_swap_to_mapping(mapping: &[usize], a: usize, b: usize) -> Vec<usize> {
    let mut new_mapping = mapping.to_vec();

    for phys in &mut new_mapping {
        if *phys == a {
            *phys = b;
        } else if *phys == b {
            *phys = a;
        }
    }

    new_mapping
}

fn mapping_is_identity(mapping: &[usize]) -> bool {
    mapping.iter().enumerate().all(|(logical, &physical)| logical == physical)
}

fn state_key(gate_index: usize, mapping: &[usize]) -> String {
    let mut s = gate_index.to_string();
    s.push('|');
    for (i, m) in mapping.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&m.to_string());
    }
    s
}

fn next_states(state: &SearchState, circuit: &Circuit, hw: &Hardware) -> Vec<SearchState> {
    let mut out = Vec::new();

    if state.gate_index < circuit.gates.len() {
        match &circuit.gates[state.gate_index] {
            Gate::H(q) => {
                let pq = state.mapping[*q];
                let mut next = state.clone();
                next.emitted.push(PhysicalGate::H(pq));
                next.gate_index += 1;
                out.push(next);
            }
            Gate::X(q) => {
                let pq = state.mapping[*q];
                let mut next = state.clone();
                next.emitted.push(PhysicalGate::X(pq));
                next.gate_index += 1;
                out.push(next);
            }
            Gate::CX(c, t) => {
                let pc = state.mapping[*c];
                let pt = state.mapping[*t];
                if has_edge(hw, pc, pt) {
                    let mut next = state.clone();
                    next.emitted.push(PhysicalGate::CX(pc, pt));
                    next.gate_index += 1;
                    next.cnot_cost += 1;
                    out.push(next);
                }

                // if not executable, try all hardware swaps
                for &(a, b) in &hw.edges {
                    let new_mapping = apply_swap_to_mapping(&state.mapping, a, b);

                    let mut next = state.clone();
                    next.mapping = new_mapping;
                    next.emitted.push(PhysicalGate::SWAP(a, b));
                    next.swap_count += 1;
                    next.cnot_cost += 3; // standard swap decomposition cost in CX basis
                    out.push(next);
                }
            }
        }
    } else {
        // logical circuit done; allow swaps to restore identity mapping
        if !mapping_is_identity(&state.mapping) {
            for &(a, b) in &hw.edges {
                let new_mapping = apply_swap_to_mapping(&state.mapping, a, b);
                let mut next = state.clone();
                next.mapping = new_mapping;
                next.emitted.push(PhysicalGate::SWAP(a, b));
                next.swap_count += 1;
                next.cnot_cost += 3;
                out.push(next);
            }
        }
    }

    out
}

fn find_solutions(
    circuit: &Circuit,
    hw: &Hardware,
    max_cnot_cost: usize,
    max_expansions: usize,
) -> Vec<Solution> {
    let start = SearchState {
        mapping: identity_mapping(circuit.num_qubits),
        gate_index: 0,
        emitted: Vec::new(),
        cnot_cost: 0,
        swap_count: 0,
    };

    let mut queue = VecDeque::new();
    queue.push_back(start);

    let mut best_seen: HashMap<String, usize> = HashMap::new();
    best_seen.insert(state_key(0, &identity_mapping(circuit.num_qubits)), 0);

    let mut solutions = Vec::new();
    let mut expansions = 0usize;

    while let Some(state) = queue.pop_front() {
        if expansions >= max_expansions {
            break;
        }
        expansions += 1;

        if state.cnot_cost > max_cnot_cost {
            continue;
        }

        if state.gate_index == circuit.gates.len() && mapping_is_identity(&state.mapping) {
            solutions.push(Solution {
                emitted: state.emitted.clone(),
                cnot_cost: state.cnot_cost,
                swap_count: state.swap_count,
            });
            continue;
        }

        for next in next_states(&state, circuit, hw) {
            if next.cnot_cost > max_cnot_cost {
                continue;
            }

            let key = state_key(next.gate_index, &next.mapping);
            let should_push = match best_seen.get(&key) {
                Some(&old_cost) => next.cnot_cost <= old_cost,
                None => true,
            };

            if should_push {
                best_seen.insert(key, next.cnot_cost);
                queue.push_back(next);
            }
        }
    }

    solutions.sort_by_key(|s| (s.cnot_cost, s.swap_count, s.emitted.len()));
    solutions
}

fn zero_state(n: usize) -> Vec<Complex64> {
    let dim = 1usize << n;
    let mut v = vec![Complex64::new(0.0, 0.0); dim];
    v[0] = Complex64::new(1.0, 0.0);
    v
}

fn apply_h(state: &mut [Complex64], q: usize, n: usize) {
    let stride = 1usize << q;
    let span = stride << 1;
    let scale = 1.0 / 2.0_f64.sqrt();

    let dim = 1usize << n;
    let mut base = 0usize;
    while base < dim {
        for i in 0..stride {
            let i0 = base + i;
            let i1 = i0 + stride;
            let a = state[i0];
            let b = state[i1];
            state[i0] = (a + b) * scale;
            state[i1] = (a - b) * scale;
        }
        base += span;
    }
}

fn apply_x(state: &mut [Complex64], q: usize, n: usize) {
    let stride = 1usize << q;
    let span = stride << 1;
    let dim = 1usize << n;

    let mut base = 0usize;
    while base < dim {
        for i in 0..stride {
            let i0 = base + i;
            let i1 = i0 + stride;
            state.swap(i0, i1);
        }
        base += span;
    }
}

fn apply_cx(state: &mut [Complex64], control: usize, target: usize, n: usize) {
    let dim = 1usize << n;

    for i in 0..dim {
        let control_bit = (i >> control) & 1;
        let target_bit = (i >> target) & 1;
        if control_bit == 1 && target_bit == 0 {
            let j = i | (1usize << target);
            state.swap(i, j);
        }
    }
}

fn apply_swap(state: &mut [Complex64], a: usize, b: usize, n: usize) {
    if a == b {
        return;
    }

    let dim = 1usize << n;
    for i in 0..dim {
        let abit = (i >> a) & 1;
        let bbit = (i >> b) & 1;
        if abit == 0 && bbit == 1 {
            let j = i ^ ((1usize << a) | (1usize << b));
            state.swap(i, j);
        }
    }
}

fn simulate_logical(circuit: &Circuit) -> Vec<Complex64> {
    let mut state = zero_state(circuit.num_qubits);

    for gate in &circuit.gates {
        match *gate {
            Gate::H(q) => apply_h(&mut state, q, circuit.num_qubits),
            Gate::X(q) => apply_x(&mut state, q, circuit.num_qubits),
            Gate::CX(c, t) => apply_cx(&mut state, c, t, circuit.num_qubits),
        }
    }

    state
}

fn simulate_physical(num_qubits: usize, gates: &[PhysicalGate]) -> Vec<Complex64> {
    let mut state = zero_state(num_qubits);

    for gate in gates {
        match *gate {
            PhysicalGate::H(q) => apply_h(&mut state, q, num_qubits),
            PhysicalGate::X(q) => apply_x(&mut state, q, num_qubits),
            PhysicalGate::CX(c, t) => apply_cx(&mut state, c, t, num_qubits),
            PhysicalGate::SWAP(a, b) => apply_swap(&mut state, a, b, num_qubits),
        }
    }

    state
}

fn states_close(a: &[Complex64], b: &[Complex64], eps: f64) -> bool {
    if a.len() != b.len() {
        return false;
    }

    a.iter()
        .zip(b.iter())
        .all(|(x, y)| (*x - *y).norm() <= eps)
}

fn print_physical_circuit(sol: &Solution) {
    for (i, gate) in sol.emitted.iter().enumerate() {
        match gate {
            PhysicalGate::H(q) => println!("{i:>3}: h p[{q}]"),
            PhysicalGate::X(q) => println!("{i:>3}: x p[{q}]"),
            PhysicalGate::CX(c, t) => println!("{i:>3}: cx p[{c}], p[{t}]"),
            PhysicalGate::SWAP(a, b) => println!("{i:>3}: swap p[{a}], p[{b}]"),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("usage: cargo run -- circuits/threeQubit.qasm");
        std::process::exit(1);
    }


    let qasm_path = Path::new(&args[1]);
    if qasm_path.extension().and_then(|s| s.to_str()) != Some("qasm") {
        eprintln!("error: expected a .qasm file");
        std::process::exit(1);
    }

    let circuit = match parse_qasm_file(qasm_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("parse error: {e}");
            std::process::exit(1);
        }
    };

    println!("parsed logical circuit:");
    println!("  num_qubits = {}", circuit.num_qubits);
    println!("  num_gates  = {}", circuit.gates.len());

    let hw = line_hardware(circuit.num_qubits);
    println!("target hardware: {}-qubit line", hw.num_qubits);
    println!("edges: {:?}", hw.edges);

    let original_cnot_count = circuit
        .gates
        .iter()
        .filter(|g| matches!(g, Gate::CX(_, _)))
        .count();
    println!("logical cx count = {original_cnot_count}");

    let max_cnot_cost = 20;
    let max_expansions = 100_000;

    let solutions = find_solutions(&circuit, &hw, max_cnot_cost, max_expansions);

    if solutions.is_empty() {
        println!("no solutions found within limits");
        return;
    }

    let logical_state = simulate_logical(&circuit);

    println!("\nfound {} candidate compiled circuits\n", solutions.len());

    for (idx, sol) in solutions.iter().take(10).enumerate() {
        let compiled_state = simulate_physical(circuit.num_qubits, &sol.emitted);
        let eq = states_close(&logical_state, &compiled_state, 1e-9);

        println!("solution #{idx}");
        println!("  cnot_cost = {}", sol.cnot_cost);
        println!("  swap_count = {}", sol.swap_count);
        println!("  equivalent = {}", eq);
        print_physical_circuit(sol);
        println!();
    }

    if let Some(best) = solutions.first() {
        println!("best solution summary:");
        println!("  cnot_cost = {}", best.cnot_cost);
        println!("  swap_count = {}", best.swap_count);
    }
}
