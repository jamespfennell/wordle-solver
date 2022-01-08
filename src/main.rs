use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;
use std::fmt::Display;

// TODO: only make guesses that are valid aka --hard-mode
// TODO: support guessing from the bigger corpus

fn main() {
    println!("Wordle Solver");
    let valid_answers = parse_words(include_str!("valid_answers.txt"));
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        1 => {
            println!("An input word is required");
            std::process::exit(1);
        }
        2 => {
            let solution_raw: Vec<char> = args[1].chars().collect();
            if solution_raw.len() != 5 {
                println!("The input word must have exactly 5 letters");
                std::process::exit(1);
            }
            let solution = Word(solution_raw[0..5].try_into().unwrap());
            // TODO: check that the solution is in the words set

            println!("\nSimulating game play for solution '{}'", solution);
            let guesser = KnownSolutionGuesser { solution };
            solve(guesser, valid_answers.clone(), valid_answers);
        }
        _ => {
            println!("Too many inputs passed");
            std::process::exit(1);
        }
    }
}

fn solve<T: Guesser>(guesser: T, allowed_guesses: Vec<Word>, mut possible_solutions: Vec<Word>) {
    let mut responses = Vec::new();
    loop {
        println!("");
        let (best, best_score) = best_guess(&allowed_guesses, &possible_solutions);
        println!(
            "Guess: {} (expected remaining solutions: {})",
            best, best_score
        );
        let response = guesser.guess(best);
        responses.push(response);
        println!("Response: {}", response);
        if response == GuessResponse::solved() {
            break;
        }
        let hist = build_histogram(best, &possible_solutions);
        possible_solutions = hist[&response].iter().map(|w| (*w).clone()).collect();
        println!("{} remaining solutions", possible_solutions.len());
    }

    println!("");
    println!("Solution: {}", possible_solutions[0]);
    for response in responses {
        println!("{}", response);
    }
}

trait Guesser {
    fn guess(&self, word: Word) -> GuessResponse;
}

struct KnownSolutionGuesser {
    solution: Word,
}

impl Guesser for KnownSolutionGuesser {
    fn guess(&self, guess: Word) -> GuessResponse {
        let f = |i| {
            if guess.0[i] == self.solution.0[i] {
                return GuessResponseChar::Exact;
            }
            for j in 0..5 {
                if guess.0[i] == self.solution.0[j] {
                    return GuessResponseChar::Included;
                }
            }
            GuessResponseChar::Excluded
        };
        GuessResponse([f(0), f(1), f(2), f(3), f(4)])
    }
}

struct InteractiveGuesser;

impl Guesser for InteractiveGuesser {
    fn guess(&self, guess: Word) -> GuessResponse {
        let f = |i| GuessResponseChar::Excluded;
        GuessResponse([f(0), f(1), f(2), f(3), f(4)])
    }
}

fn best_guess(guesses: &[Word], possible_solutions: &[Word]) -> (Word, f64) {
    if possible_solutions.len() == 1 {
        return (possible_solutions[0], 1.0);
    }
    let mut best = guesses[0];
    let mut best_score = usize::MAX;
    for guess in guesses {
        let hist = build_histogram(*guess, possible_solutions);
        let mut expected_remaining = 0;
        for val in hist.values() {
            expected_remaining += val.len() * val.len();
        }
        if expected_remaining < best_score {
            best = *guess;
            best_score = expected_remaining;
        }
    }
    (
        best,
        (best_score as f64) / (possible_solutions.len() as f64),
    )
}

fn build_histogram<'a>(
    guess: Word,
    possible_solutions: &[Word],
) -> HashMap<GuessResponse, HashSet<&Word>> {
    let mut hist = HashMap::default();
    for solution in possible_solutions {
        let response = KnownSolutionGuesser {
            solution: *solution,
        }
        .guess(guess);
        hist.entry(response)
            .or_insert(HashSet::default())
            .insert(solution);
    }
    hist
}

fn parse_words(words_txt: &str) -> Vec<Word> {
    let mut words = Vec::with_capacity(words_txt.len() / 6);
    let mut pos = 0;
    let mut letters = ['0'; 5];
    for c in words_txt.chars() {
        if pos == 5 {
            if c != '\n' {
                panic!("Bad file")
            }
            words.push(Word::new(letters));
            pos = 0;
            continue;
        }
        letters[pos] = c;
        pos += 1;
    }
    words
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
struct Word([char; 5]);

impl Word {
    fn new(chars: [char; 5]) -> Word {
        Word(chars)
    }
}

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}{}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4],
        )
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
struct GuessResponse([GuessResponseChar; 5]);

impl GuessResponse {
    fn solved() -> GuessResponse {
        GuessResponse([GuessResponseChar::Exact; 5])
    }
}

impl Display for GuessResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}{}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4],
        )
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
enum GuessResponseChar {
    /// The character is not in the answer
    Excluded,
    /// The character is in the answer at the exact position provided
    Exact,
    /// The character is in the answer but at a different position than provided
    Included,
}

impl GuessResponseChar {
    fn unicode_char(&self) -> char {
        match self {
            GuessResponseChar::Excluded => '⬛',
            GuessResponseChar::Exact => '🟩',
            GuessResponseChar::Included => '🟨',
        }
    }
}

impl Display for GuessResponseChar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.unicode_char(),)
    }
}
