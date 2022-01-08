use std::fmt::Display;
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;

// TODO: recursively calculate the probabilies for subsequently choices --recursive
// TODO: only make guesses that are valid aka --hard-mode
// TODO: if there is only 1 solution in each bucket, go into hard mode?

fn main() {
    println!("Wordle Solver");
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("An input word is required");
        std::process::exit(1);
    }
    let solution_raw: Vec<char> = args[1].chars().collect();
    if solution_raw.len() != 5 {
        println!("The input word must have exactly 5 letters");
        std::process::exit(1);
    }
    let solution = Word(solution_raw[0..5].try_into().unwrap());
    let guesser = KnownWordGuesser{
        word: &solution,
    };

    let valid_answers = include_str!("valid_answers.txt");
    let words = parse_words(valid_answers);
    let mut possible_solutions = words.clone();
    while possible_solutions.len() > 1 {
        let (best, best_score) = best_guess(&words, &possible_solutions);
        println!("Guess {} (expected remaining solutions: {})", best, best_score);
        let response = guesser.guess(best);
        let hist = build_histogram(best, &possible_solutions);
        possible_solutions = hist[&response].iter().map(|w| {(*w).clone()}).collect();
        println!("{} remaining solutions", possible_solutions.len());
    }
    println!("Solution: {}", possible_solutions[0]);
}

trait Guesser {
    fn guess(&self, word: &Word) -> GuessResponse;
}

struct KnownWordGuesser<'a> {
    word: &'a Word,
}

impl<'a> Guesser for KnownWordGuesser<'a> {
    fn guess(&self, guess: &Word) -> GuessResponse {
        let f = |i| {
            if guess.0[i] == self.word.0[i] {
                return GuessResponseChar::Exact;
            }
            for j in 0..5 {
                if guess.0[i] == self.word.0[j] {
                    return GuessResponseChar::Included;
                }
            }
            GuessResponseChar::Excluded
        };
        GuessResponse([f(0), f(1), f(2), f(3), f(4)]) 
    }
}

fn best_guess<'a>(guesses: &'a [Word], possible_solutions: &[Word]) -> (&'a Word, usize) {
    let mut best = &guesses[0];
    let mut best_score = possible_solutions.len() * possible_solutions.len();
    for guess in guesses {
        let hist = build_histogram(guess, possible_solutions);
        let mut expected_remaining = 0;
        for val in hist.values() {
            expected_remaining += val.len() * val.len();
        }
        if expected_remaining < best_score {
            best = guess;
            best_score = expected_remaining;
        }
    }
    (best, best_score / possible_solutions.len())
}

fn build_histogram<'a>(guess: &Word, possible_solutions: &'a [Word]) -> HashMap<GuessResponse, HashSet<&'a Word>> {
    let mut hist = HashMap::default();
    for solution in possible_solutions {
        let response = KnownWordGuesser {
            word: solution,
        }.guess(guess);
        hist.entry(response).or_insert(HashSet::default()).insert(solution);
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

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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


#[derive(Hash, PartialEq, Eq)]
struct GuessResponse([GuessResponseChar; 5]);

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
enum GuessResponseChar {
    /// The character is not in the answer
    Excluded,
    /// The character is in the answer at the exact position provided
    Exact,
    /// The character is in the answer but at a different position than provided
    Included,
}
