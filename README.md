# Rustsolver

a simple dpll algorithm with basic heuristics

## Benchmark

![Benchmark](cactus_plot.png) Cactus plot with one minute of cpu time for each problem.

| Heuristic    | Solved | Timeout | Percentage Solved |
|--------------|--------|---------|-------------------|
| VSIDS        | 146    | 26      | 84.88%            |
| None         | 112    | 60      | 65.11%            |
| MOM          | 111    | 61      | 64.53%            |
| JeroslowWang | 110    | 62      | 63.90%            |
| DLIS         | 98     | 74      | 56.97%            |
| DLCS         | 97     | 75      | 56.39%            |

### None

```bash
[2024-01-18T12:20:02Z INFO  benchmark] Heuristic: None
[2024-01-18T12:20:02Z INFO  benchmark] Solved: 112
[2024-01-18T12:20:02Z INFO  benchmark] Timeout: 60
[2024-01-18T12:20:02Z INFO  benchmark] Error: 0
[2024-01-18T12:20:02Z INFO  benchmark] Total: 172
[2024-01-18T12:20:02Z INFO  benchmark] Solved: 65.11627906976744%
```

### DLIS

```bash
[2024-01-18T12:22:10Z INFO  benchmark] Heuristic: DLIS
[2024-01-18T12:22:10Z INFO  benchmark] Solved: 98
[2024-01-18T12:22:10Z INFO  benchmark] Timeout: 74
[2024-01-18T12:22:10Z INFO  benchmark] Error: 0
[2024-01-18T12:22:10Z INFO  benchmark] Total: 172
[2024-01-18T12:22:10Z INFO  benchmark] Solved: 56.97674418604651%
```

### DLCS

```bash
[2024-01-18T12:24:20Z INFO  benchmark] Heuristic: DLCS
[2024-01-18T12:24:20Z INFO  benchmark] Solved: 97
[2024-01-18T12:24:20Z INFO  benchmark] Timeout: 75
[2024-01-18T12:24:20Z INFO  benchmark] Error: 0
[2024-01-18T12:24:20Z INFO  benchmark] Total: 172
[2024-01-18T12:24:20Z INFO  benchmark] Solved: 56.395348837209305%
```

### MOM

```bash
[2024-01-18T15:32:42Z INFO  benchmark] Heuristic: MOM
[2024-01-18T15:32:42Z INFO  benchmark] Solved: 111
[2024-01-18T15:32:42Z INFO  benchmark] Timeout: 61       
[2024-01-18T15:32:42Z INFO  benchmark] Error: 0
[2024-01-18T15:32:42Z INFO  benchmark] Total: 172
[2024-01-18T15:32:42Z INFO  benchmark] Solved: 64.53488372093024%
```

### JeroslowWang

```bash
[2024-01-18T15:34:44Z INFO  benchmark] Heuristic: JeroslowWang
[2024-01-18T15:34:44Z INFO  benchmark] Solved: 110
[2024-01-18T15:34:44Z INFO  benchmark] Timeout: 62
[2024-01-18T15:34:44Z INFO  benchmark] Error: 0
[2024-01-18T15:34:44Z INFO  benchmark] Total: 172
[2024-01-18T15:34:44Z INFO  benchmark] Solved: 63.95348837209303%
```

### VSIDS

```bash
[2024-01-18T15:36:02Z INFO  benchmark] Heuristic: VSIDS
[2024-01-18T15:36:02Z INFO  benchmark] Solved: 146
[2024-01-18T15:36:02Z INFO  benchmark] Timeout: 26
[2024-01-18T15:36:02Z INFO  benchmark] Error: 0
[2024-01-18T15:36:02Z INFO  benchmark] Total: 172
[2024-01-18T15:36:02Z INFO  benchmark] Solved: 84.88372093023256%
```

## setup

### debug build

```bash
cargo build
```

### release build

```bash
cargo build --release
```

## usage

directly over cargo or use the build binary

### binary

```bash
./target/release/dpll -h

Usage: dpll <COMMAND>

Commands:
  test       run the test function
  tests      run the tests on the given directory
  benchmark  runs the benchmark on the given directory, uses all of your cpu power
  solve      solve the given cnf file
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### solve

```bash
./target/release/dpll solve -h
Usage: dpll solve <FILE> [HEURISTIC]

Arguments:
  <FILE>       The file to run
  [HEURISTIC]  The heuristic to use [possible values: none, mom, dlis, dlcs, jeroslow-wang]

Options:
  -h, --help  Print help
```

### cargo

```bash
cargo run --release -- -h
Usage: dpll <COMMAND>

Commands:
  test       run the test function
  tests      run the tests on the given directory
  benchmark  runs the benchmark on the given directory, uses all of your cpu power
  solve      solve the given cnf file
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

#### solve

```bash
cargo run --release -- solve -h
Usage: dpll solve <FILE> [HEURISTIC]

Arguments:
  <FILE>       The file to run
  [HEURISTIC]  The heuristic to use [possible values: none, mom, dlis, dlcs, jeroslow-wang]

Options:
  -h, --help  Print help
```

## Log

set the RUST_LOG environment variable to get log to sdtout. For debug use
`RUST_LOG=debug` this make the program really slow so use it only for debuging. If you want to se the time it took to
solve the cnf use
`RUST_LOG=info`

## example

```bash
RUST_LOG=info cargo run --release -- solve ./data/inputs/sat/aim-50-1_6-yes1-1.cnf jeroslow-wang

[2024-01-18T11:45:13Z INFO  dpll] solved in 14.720015ms
s SATISFIABLE
v -1 2 3 -4 -5 -6 7 8 9 -10 -11 -12 -13 14 -15 -16 17 18 19 20 21 22 23 24 -25 26 27 28 -29 30 31 -32 -33 -34 35 36 -37 38 39 40 41 42 43 -44 -45 46 -47 48 -49 -50
```

```bash
cargo run --release -- solve ./data/inputs/sat/aim-50-1_6-yes1-1.cnf jeroslow-wang

s SATISFIABLE
v -1 2 3 -4 -5 -6 7 8 9 -10 -11 -12 -13 14 -15 -16 17 18 19 20 21 22 23 24 -25 26 27 28 -29 30 31 -32 -33 -34 35 36 -37 38 39 40 41 42 43 -44 -45 46 -47 48 -49 -50
```

```bash
RUST_LOG=info ./target/release/dpll solve ./data/inputs/sat/aim-50-1_6-yes1-1.cnf jeroslow-wang

[2024-01-18T11:45:13Z INFO  dpll] solved in 14.720015ms
s SATISFIABLE
v -1 2 3 -4 -5 -6 7 8 9 -10 -11 -12 -13 14 -15 -16 17 18 19 20 21 22 23 24 -25 26 27 28 -29 30 31 -32 -33 -34 35 36 -37 38 39 40 41 42 43 -44 -45 46 -47 48 -49 -50
```

```bash
./target/release/dpll solve ./data/inputs/sat/aim-50-1_6-yes1-1.cnf jeroslow-wang

s SATISFIABLE
v -1 2 3 -4 -5 -6 7 8 9 -10 -11 -12 -13 14 -15 -16 17 18 19 20 21 22 23 24 -25 26 27 28 -29 30 31 -32 -33 -34 35 36 -37 38 39 40 41 42 43 -44 -45 46 -47 48 -49 -50
```
