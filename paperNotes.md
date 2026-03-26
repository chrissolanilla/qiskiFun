# lightSabre Paper notes

# abstract notes
## improvements made the SABRE algorithm 200 times faster.
## before lightSabre, qiskit was using 0.20.1
they used release valvue mechanism in this version already.
was not in rust yet.

## Li Et al decreases swap gate counts by 18.9 percent across the same benchmarks

## Sabre sturggles with scalability and convergence(what?) on large circuits.
## concludes with it being a good soution for now and the future hardware.

# introduction notes
## SABRE algo published in 2018
## was state of the art for quality circuits at reasonable runtime.
## recently the run time has been a concern with SABRE
## it did not originally support control flow(Section 5. control flow).
## paper proclaims to detail modifications on Qiskit to address runtime and quality along with other stuff
## cites 5 different papers on improving SABRE algorithm focusing on circuit quality.
    first one is :  keeps track of multiple candidates circuits during routing and continuously
                    adapts this set by replacing worse candidates by better ones;

    second one is: considers distancetwo bridge gates in addition to swap gates;

    third one is: presents a look-ahead heuristic that improves the circuit size;

    fourth one: describes a scheme to reduce the circuit depth/execution time;

    fifth: combines routing and synthesis.

## notes that for these above works, they have a substantial runtime cost tradeoff for quality.
    impractical for large scale circuits.

## for focusing on runtime, lightSabre is implemented in rust
## they use relative scoring mechanism for circuit quality to evaluate heuristic swaps in O(1) time.
        MORE ABOUT THIS EXPLAINED IN SECTION II

## mentions how they exploit the nonDestrministic behavior of SABRE to improve quality?
    THIS ACTUALLY I READ STRAGIHT TO THE SECOND COUMN OF IMPROVEMNTS
    explains that the quality of sabre is largely influence by initial equation for the system
        after a candidate swap has been made? and then it selects randomly from the set of swaps
            induce lowest heuristic value.

## expalins that a swap is a candidate if it involves atleast one qubit that is an operand of a
    frontlayer gate(what is a frontlayer gate?)

## some very confusing stuff about hardware-topology families and how they have finite average connectivity?
    What is finite average connectivity? no matter dimension?
    rings have node of degree 2, periodic grids have node of degree 4. heavy hex lattice have degrees up to 3.
    The number of candidate swaps is, therefore, typically proportional to |F|(what is f? magnitude of f?).
    the computational complexity of choosing a “best” swap by this method is Θ(|F|^2)
    this is all assuming the max size of the extended set is at most some constant proportion to max front layer size.

## applying a single swap affects a maximum of 2 + |E| terms in eq. (1) (what is E? and edge?)
## it hink basiaclly we have the front and the extended layer. We analze the front and the extended is everythign else
## if we do swaps on the front then it can affect everything in the extend
## the paper notes that when you do swasp in the extended layer, the run time becomes O(1) instead.
    This is because the extended layer is constranied to be some qubit count independent size and not varibale.



OK LETS START OVER WITH IMPROVEMNTS STAERTING.

ok one last bit about the shit. Sabre runs multiple trials and selects the lowest swaps.
this makes the runtime particually crucial.
It directly impacts the feasibility of running multiple trials of the algorithm and achieving high-quality results
sabre has the ability to have seed the layout for mroe than random mapping
Apparently lightSabre has a release valve mechanism which is for SABRE og problem of getting stuck on lookahead heuristic.

## introduciton ending: need to support hardawre with multiple disjoint components(what this mean?)
    also control flow and feedforward. discuessed in seciton II 4 and 5.

# IMPROVEMENTS
 ### relative scoring
## the sabre heutristic has the basic component, and the lookahead component
the equation 1 is very nicely labled.
H(heuristic i guess) = 1/(|F|) * Sum of dist(i,j) + (k/|E|) * Sum of dist(i,j)
F and E are gates int he front and extended layer. K is a relative weight chosen by the implementor.
The two sums are over the pairs of physical qubits whose virtual qubits partake in a gate in the relveant set.(??)
    I understand physical vs virtual qubits, but the gate in the revelant set? need more clarification

The function dist(i, j) counts the distance between physical qubits i and j.
if two physical qubits can interact, they have a distance of unity(unitary? unit 1?)

## the heurisitc is a scoring for the total system of the front layer and extended set on the assumtion that
    a lookahead table for dist is precalculated. (it only requries hardware topology to be known)
        its kind of like the graph of dist like the prefix sum thing

## if not gates from the front layer are routable(no solution?) SABRE iterates over candidate swaps.
It will evalute the heuristic in equation 1 for the system after the swap candidate has been made and
select randomly over the swaps that induce lowest heuristic value.

### multiple trials
## stochastic elements?(whats stochastistc?(apparently means random probability spread?))
## there are two times in teh SABRE algo where there are stochastic elements:
    once in the inital layout being randomly seelcted, and during routing.
    in routing, if there are multiple candidates that are equally good(minimum scores(less number of swaps? or heustric score?))
        then the swap is randomly selected.

## this made SABRE very dependant on the RNG. lightSabre tries a different approach with multiple trials and seeds in parallel.
    out of all the trials, the one with the fewest swap gates is chosen.

#### there are 2 graphs shown to me about QFT which is quantum fourier transform.
showing basically that with more trials, the swap gate counts get reduced consitently.
however, the second graph says taht when graphing to show the circuit depth, sometimes it dosent always follow.
since the depth can get more with less swaps somehow.

## apparently this parallel thing dosent actually get us the best layout for layout purposes,
    For this reason LightSABRE only runs a single routing trial when
        running routing as part of layout, but will run multiple
        routing trials after a layout has been initialized.
        not sure what that meant tbh.

## they try to say that when they run the multiple trials in parallel, the layout becomes better for less swaps
    good cause the runtime cost becomes managable since it is less order.


### seeding the initial layout
we need to understand that there are two phases which are layout and routing.
The part that happens in parallel is choosing a layout. When we do the multiple layouts in paralle and choose the best one,
we only have one routing trial and then compare the best one to choose that one.

AFTER we choose the layout, is when we do the multiple trials for routing for that one Layout.
Notablty, it is only one random trial by default, but it is optinal to add an additional route if needed fo rmore optimal circuits.

## before those two phases, qiskit has tools for us as analysis tools to do before that.
One of them is the VF2Layout analysis pass which checks if the circuit can be perfectly embedded on the hardware
connectivity map by solving the subgraph isomorphism problem [9].
They give an example of how a ring map of like 20 qubits can be perfect on a heavy-hex topology
(when we say topology, are we saying its the connectivity map? or the hardware topology(what?)?).
yeah its basically that. Heavy-hex is the ibm one.

for "perfect" vs "almost perfect" its like perfect means no swaps or like the connection is there.
    almost perfect is like a couple of swaps needed at most. can "embed" everyhting almost perf

## SabrePreLayout
SabrePreLayout works by getting the original connectivity graph and adding additonal edges of some distnace d
d is usually 2. It uses rustworkx which is a graph library for python built in rust for handling complex graphs.
it is said to solve the "isomorphism problem"(just seeing if two graphs are the same(subgraphs too)).
rustworkx checks if it exists.

## figure 2
the figure shows the progression from bad, to better, to best from SABRE(A), to SabrePreLayout(B) and
improved SabrePreLayout(C).
Baiscally the goal is to map the red virtual qubits onto the blue physical qubits.
the arrows indicate the distance of the physical qubits. B made it up to only 2, but C is better.

They say that the sabrePRelaouy can be optimized further for circuit quality by solving sub graphs
like divide and conquer i guess.
As device circuits grow in size, the random layout tends to detiriot.
#### improving the selection of starting layouts is an area of active ongoing research [11].

### Disjoint connectivity graphs
Before, SABRE assumed that the connectivity map would be fully connected.
However, recent vendors introduced modular QPUs which are not. Therefore we need some sort of transpiler for
disjointed graphs. (QPUs with sharing classical resources with no quantum connectivity.)
another scenario this can occur is when there are faulty qubits. so its important.

## to support this lightSabre does initial analysis in the layout and routing problem
first they analyzed the connected parts of the graph
if there are more than one island, then they want to identify that.
they then make DAG representations of the circuit. it has to be acyclical because you cant have some
hardware depend on itself. its meant to show who needs to talk to who.
A GREEDY placement algorithm is used to map each independent circuit component (circuit island)
onto a hardware island (connected component in the connectivity graph), ensuring that
all required interactions happen within reachable qubits.

Routing hsould be done after the layout is chosen. Doing it on isolated componets(islands) can yield incorrect results
THe layout happens serprately on each island.

### Control Flow
Some circuits want to do measurements in the middle of circuit, and then branch off depending on the measurement
SABRE didnt really support this. lightSabre does though. this is called Control Flow

## In the original SABRE algorithm, the main loop adds gates to the front layer as they become executable
the implications of having multiple islands means that the routing has to happen seperately for each island
since each island is basically its own cicuit.

## figure 3 talks about how there are classical bits and quantum bits and shows the DAG of the if-else.
the yliket o emphasize its decided at runtime the if else on appying h gate or x gate.
the process of writing down measured values into classical bits for later quantum circuits is called feed forward.

## control flow and lookahead
lookahead refers to the extended layer in SABRE, which considers future gates when evaluating swaps.
control flow breaks this model because it introduces multiple possible future paths instead
of a single deterministic sequence of gates.

these operations do not fit naturally in the front layer since it is unclear which branch will execute at runtime.

lightSabre resolves this by:
- routing each branch separately
- adding SWAPs (epilogue) at the end of each branch to align them to a common layout

this ensures that future gates see a consistent layout regardless of which branch executes,
allowing the lookahead heuristic to function normally. as a result, control flow can be treated
similarly to a simple gate from the perspective of the outer circuit.

### Heuristic Enhancements
They want to improve the heuristic by introducing 2 things: depth and critical path.
The weights of each heuristic component can be chosen to be constant,
    or something that scales iwth the size of the set

    Each of those components support relative scoring offering further customization.
    these are independant so you can have combinations like depth and critical path, or just one of them.

## When should a user optimize for depth or critical path???

## depth component: a heuristic that aims to reduce the total depth of the circuit
IT is defined by the quation of (D*delta_depth)/3
D is the weight, and the delta_depth is the difference between the depth of the current circuit and the one after swap
the depth here is from 2 qubit gate depth, and the 3 is cause of the 3 CNOTS that make a swap.

## the depht heuristic componetn is good for reducing depth, but introudces more runtime.
it has to track the qubit depth after each swap candidate and compute the true depth impact including
subsequent routable gates.

## critical path component:
We can assign the abstract circuit a critical path to add to the heuristic.
with this, it will keep in mind the ranking of paths that are critical with 1 being the highest rank.
the equation is defined as alpha^(r_rank). alpha is a value between 0 and 1 and r_rank is the rank of the path.
it tracks the number of descendants of each gate and assigns them a ranking,
allowing the algorithm to prioritize gates that are critical to the circuit’s execution.
## IN GENERAL it is not an all too great heuristic, but for certain critical path circuits it is good.
what would be a good example vs a bad example?

## Summary, the choice of heuristic should be implementation specific for the circuit.
figure 4 a shows of a 127 hex heavy qubit hardawre showcasing that the lookahead and decay heuristic is the best choice
for circuit depth, obviously the depth heursitc is the best for lowest depth, intrestingly lookahead and decay is the worst.

they mention depth is the best overall good choice for average deptha nd swapcounts.

## QV Circuits
Quantum volume circuits: circuits where the number of qubits is equal to the circuit depth.
They did benchmarks on quantum volumes mainly for robustness of heuristic enhancements.
## lookahead and decay generally offered best swap reductions, but critical path was good for reducing depth but only
    for circuits with a cirtical path clearly visisible.

SABRE heuristic can get stuck in local minima
Best-scoring swaps may make no progress
LightSABRE detects when it's stuck:

too many swaps without routing a gate

LightSABRE reduces CNOT (swap) count
Average improvement:
~18.9% fewer gates
Better quality comes from:
Multiple trials
Better heuristics

Runtime scales well with number of qubits
Tested up to ~20,000 qubits
Swap count grows reasonably
