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
//hardware constructors
fn line_hardware(n: usize) -> Hardware {
    let mut edges = Vec::new();
    for i in 0..n.saturating_sub(1) {
        edges.push((i, i + 1));
    }
    Hardware { num_qubits: n, edges }
}

fn star_hardware(n: usize, center: usize) -> Hardware {
    let mut edges = Vec::new();
    for q in 0..n {
        if q != center {
            edges.push((center, q));
        }
    }
    Hardware { num_qubits: n, edges }
}

fn custom_five_qubit_hardware() -> Hardware {
    Hardware {
        num_qubits: 5,
        edges: vec![
            (0, 1),
            (0, 2),
            (0, 3),
            (2, 4),
        ],
    }
}

fn get_flag_value(args: &[String], flag: &str) -> Option<String> {
    for i in 0..args.len() {
        if args[i] == flag {
            return args.get(i + 1).cloned();
        }
    }
    None
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

fn gate_row(qubit: usize) -> usize {
    qubit * 2
}

fn connector_row_between(a: usize, b: usize) -> std::ops::RangeInclusive<usize> {
    let low = a.min(b);
    let high = a.max(b);
    (gate_row(low) + 1)..=(gate_row(high) - 1)
}

fn blank_column(num_qubits: usize, cell_width: usize) -> Vec<String> {
    let total_rows = num_qubits * 2 - 1;
    vec!["-".repeat(cell_width); total_rows]
}

fn centered_cell(cell_width: usize, label: &str, fill: char) -> String {
    let label_width = label.chars().count();
    let left = (cell_width.saturating_sub(label_width)) / 2;
    let right = cell_width.saturating_sub(label_width + left);

    let mut s = String::new();
    s.push_str(&fill.to_string().repeat(left));
    s.push_str(label);
    s.push_str(&fill.to_string().repeat(right));
    s
}

fn vertical_cell(cell_width: usize) -> String {
    let mid = cell_width / 2;
    let mut s = String::new();

    for i in 0..cell_width {
        if i == mid {
            s.push('|');
        } else {
            s.push(' ');
        }
    }

    s
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
            //not need for unitary equivalence
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

        if line.starts_with("U(pi/2, 0, pi)") {
            let q = parse_index(line).ok_or("bad U(pi/2, 0, pi) gate")?;
            gates.push(Gate::H(q));
            continue;
        }

        if line.starts_with("U(pi, 0, pi)") {
            let q = parse_index(line).ok_or("bad U(pi, 0, pi) gate")?;
            gates.push(Gate::X(q));
            continue;
        }

        return Err(format!("unsupported line: {line}"));
    }

    if num_qubits == 0 {
        return Err("did not find qubit declaration".to_string());
    }

    Ok(Circuit { num_qubits, gates })
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

                //try hardware swaps if not execuatble
                for &(a, b) in &hw.edges {
                    let new_mapping = apply_swap_to_mapping(&state.mapping, a, b);

                    let mut next = state.clone();
                    next.mapping = new_mapping;
                    next.emitted.push(PhysicalGate::SWAP(a, b));
                    next.swap_count += 1;
                    //standard swap decomposition cost in CX basis
                    next.cnot_cost += 3;
                    out.push(next);
                }
            }
        }
    } else {
        //logical circuit done, now restore identiy map
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
    prune_by_best_cost: bool,
    max_steps: usize,
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
        if state.emitted.len() > max_steps {
            continue;
        }

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

            if prune_by_best_cost {
                let key = state_key(next.gate_index, &next.mapping);
                let should_push = match best_seen.get(&key) {
                    Some(&old_cost) => next.cnot_cost <= old_cost,
                    None => true,
                };

                if should_push {
                    best_seen.insert(key, next.cnot_cost);
                    queue.push_back(next);
                }
            } else {
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

fn print_logical_circuit(circuit: &Circuit) {
    println!("\n=== logical circuit ===");

    for (i, gate) in circuit.gates.iter().enumerate() {
        match gate {
            Gate::H(q) => println!("{i:>3}: h q[{q}]"),
            Gate::X(q) => println!("{i:>3}: x q[{q}]"),
            Gate::CX(c, t) => println!("{i:>3}: cx q[{c}], q[{t}]"),
        }
    }
}

fn draw_logical_circuit_wrapped(circuit: &Circuit, max_cols: usize) {
    let num_qubits = circuit.num_qubits;
    let cell_width = 7;
    let mut columns: Vec<Vec<String>> = Vec::new();

    for gate in &circuit.gates {
        let mut col = blank_column(num_qubits, cell_width);

        match gate {
            Gate::H(q) => {
                col[gate_row(*q)] = centered_cell(cell_width, "H", '-');
            }
            Gate::X(q) => {
                col[gate_row(*q)] = centered_cell(cell_width, "X", '-');
            }
            Gate::CX(c, t) => {
                col[gate_row(*c)] = centered_cell(cell_width, "o", '-');
                col[gate_row(*t)] = centered_cell(cell_width, "CX", '-');

                for r in connector_row_between(*c, *t) {
                    col[r] = vertical_cell(cell_width);
                }
            }
        }

        columns.push(col);
    }

    print_wrapped_columns(num_qubits, &columns, max_cols, "q");
}

fn draw_physical_circuit_wrapped(num_qubits: usize, gates: &[PhysicalGate], max_cols: usize) {
    let cell_width = 7;
    let mut columns: Vec<Vec<String>> = Vec::new();

    for gate in gates {
        let mut col = blank_column(num_qubits, cell_width);

        match gate {
            PhysicalGate::H(q) => {
                col[gate_row(*q)] = centered_cell(cell_width, "H", '-');
            }
            PhysicalGate::X(q) => {
                col[gate_row(*q)] = centered_cell(cell_width, "X", '-');
            }
            PhysicalGate::CX(c, t) => {
                col[gate_row(*c)] = centered_cell(cell_width, "o", '-');
                col[gate_row(*t)] = centered_cell(cell_width, "CX", '-');

                for r in connector_row_between(*c, *t) {
                    col[r] = vertical_cell(cell_width);
                }
            }
            PhysicalGate::SWAP(a, b) => {
                col[gate_row(*a)] = centered_cell(cell_width, "SW", '-');
                col[gate_row(*b)] = centered_cell(cell_width, "SW", '-');

                for r in connector_row_between(*a, *b) {
                    col[r] = vertical_cell(cell_width);
                }
            }
        }

        columns.push(col);
    }

    print_wrapped_columns(num_qubits, &columns, max_cols, "p");
}

fn print_wrapped_columns(
    num_qubits: usize,
    columns: &[Vec<String>],
    max_cols: usize,
    prefix: &str,
) {
    let total_rows = num_qubits * 2 - 1;
    let chunk_size = max_cols.max(1);

    for (chunk_idx, chunk) in columns.chunks(chunk_size).enumerate() {
        if chunk_idx > 0 {
            println!();
        }

        for row in 0..total_rows {
            if row % 2 == 0 {
                let q = row / 2;
                print!("{prefix}{q}: ");
            } else {
                print!("    ");
            }

            for col in chunk {
                print!("{}", col[row]);
            }
            println!();
        }
    }
}

fn print_hardware(hw: &Hardware) {
    println!("hardware qubits: {}", hw.num_qubits);
    println!("hardware edges:");
    for (a, b) in &hw.edges {
        println!("  {} -- {}", a, b);
    }
}

fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // eprintln!("usage: cargo run -- <file.qasm> [--all] [--diagram]");
        eprintln!("usage: cargo run -- <file.qasm> [--all] [--all-valid] [--diagram]");
        std::process::exit(1);
    }


    let show_all = args.contains(&"--all".to_string());
    let show_all_valid = args.contains(&"--all-valid".to_string());
    let show_diagram = args.contains(&"--diagram".to_string());

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

    // let hw = line_hardware(circuit.num_qubits);

    let hardware_name = get_flag_value(&args, "--hardware")
                            .unwrap_or_else(|| "line".to_string());

    let hw = match hardware_name.as_str() {
        "line" => line_hardware(circuit.num_qubits),
        "star" => star_hardware(circuit.num_qubits, 0),
        "custom5" => custom_five_qubit_hardware(),
        _ => {
            eprintln!("unknown hardware: {}", hardware_name);
            eprintln!("available hardware: line, star, custom5");
            std::process::exit(1);
        }
    };
    println!("target hardware: {}-qubit line", hw.num_qubits);
    print_hardware(&hw);
    println!("edges: {:?}", hw.edges);

    let original_cnot_count = circuit
        .gates
        .iter()
        .filter(|g| matches!(g, Gate::CX(_, _)))
        .count();
    println!("logical cx count = {original_cnot_count}");

    print_logical_circuit(&circuit);

    if show_diagram {
        println!("===original circuit===");
        println!("\nlegend: o = cnot control, CX = cnot target, SW = swap, X = pauli-x");
        // draw_logical_circuit(&circuit);
        draw_logical_circuit_wrapped(&circuit, 12);
    }

    let max_cnot_cost = 20;
    let max_expansions = 100_000;

    // let solutions = find_solutions(&circuit, &hw, max_cnot_cost, max_expansions);
    let prune_by_best_cost = !show_all_valid;
    let max_steps = 1_000_000;
    let solutions = find_solutions(
        &circuit,
        &hw,
        max_cnot_cost,
        max_expansions,
        prune_by_best_cost,
        max_steps,
    );

    if solutions.is_empty() {
        println!("no solutions found within limits");
        return;
    }

    let logical_state = simulate_logical(&circuit);


    let to_show = if show_all {
        solutions.len()
    } else {
        1
    };

    for (idx, sol) in solutions.iter().take(to_show).enumerate() {
        let compiled_state = simulate_physical(circuit.num_qubits, &sol.emitted);
        let eq = states_close(&logical_state, &compiled_state, 1e-9);

        println!("\n=== solution #{idx} ===");
        println!("cnot_cost = {}", sol.cnot_cost);
        println!("swap_count = {}", sol.swap_count);
        println!("equivalent = {}", eq);

        if show_diagram {
            // draw_physical_circuit(circuit.num_qubits, &sol.emitted);
            draw_physical_circuit_wrapped(circuit.num_qubits, &sol.emitted, 12);
        } else {
            print_physical_circuit(sol);
        }
    }
}
