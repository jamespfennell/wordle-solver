# Wordle Solver

This is a solver for the [Wordle puzzle game](https://www.powerlanguage.co.uk/wordle/).
At each stage in the game, the solver guesses the word that minimizes the expected number of
  possible solutions that will be left.

## Running the solver

The solver is written in Rust so rustc and cargo need to be installed.
It is a pretty slow in debug mode, so it's recommended to first build it in release mode:

```
cargo build --release
```

The solver has two modes.
In *interactive mode*, the solver suggests guesses and asks for the response
the Wordle game returns for those guesses.
Eventually it figures out the solution.
This mode is run using:
```
./target/release/wordle-solver
```

In *simulation mode*, the solver simulates the moves it would make given a provided
true solution.
The solution is provided via the string flag `-s` or `--solution`:
```
./target/release/wordle-solver -s crank
Wordle Solver

Simulating game play for solution 'crank'

Guess: raise (expected remaining solutions: 61.00086393088553)
Response: 🟨🟨⬛⬛⬛
78 remaining solutions

Guess: crowd (expected remaining solutions: 5.205128205128205)
Response: 🟩🟩⬛⬛⬛
5 remaining solutions

Guess: fancy (expected remaining solutions: 1)
Response: ⬛🟨🟨🟨⬛
1 remaining solutions

Guess: crank (expected remaining solutions: 1)
Response: 🟩🟩🟩🟩🟩

Solution: crank
🟨🟨⬛⬛⬛
🟩🟩⬛⬛⬛
⬛🟨🟨🟨⬛
🟩🟩🟩🟩🟩
```

The flag `--hard-mode` runs the solver with Wordle's hard mode rules in which
  only guesses satisfying the known constraints may be provided.

## The solver algorithm

At each stage in the game there is a set of possible solutions, starting with 
[Wordle's 2315 valid solutions](https://github.com/jamespfennell/wordle-solver/blob/main/src/valid_solutions.txt).
After making a specific guess, there are up to `3^5=243` possible responses that Wordle can return.
Each character can either be 🟩 (exact match), 🟨 (non-exact match) or ⬛ (no match), and there are 5 letters, hence `3^5`.
Note that 243 is an upper bound on the number of responses: if there are only 5 solutions left, Wordle can return at most 5 different responses.

Given a specific guess, we can *partition* the set of solutions by grouping together 
  solutions that return the same response from Wordle.
For example, if the possible solutions are 
  `banal`, `canal` and `world` and we guess `final` the partitions are:

- ⬛⬛🟩🟩🟩: `banal`, `canal`
- ⬛⬛⬛🟨⬛: `world`

The probability that the first response is returned is two thirds; and the second response one third.
In general (assuming a uniform distribution of solutions) the probability that a particular response will
  be returned by Wordle is proportional to the number of solutions that generate that response.

Finally, we use the partition probabilities to calculate the expected number of solutions that will be left.
This is simply `(probability of partition) * (number of solutions in partition)` summed over all partitions.

## Credit

The initial implementation of this idea was O(N^3) where N is the number of possible solutions.
[Tom O'Neill's blog post](https://notfunatparties.substack.com/p/wordle-solver/)
  taught me that there was an equivalent O(N * max(N, 243)) algorithm, 
  which is used for the current implementation.
