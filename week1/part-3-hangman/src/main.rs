// Simple Hangman Program
// User gets five incorrect guesses
// Word chosen randomly from words.txt
// Inspiration from: https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html
// This assignment will introduce you to some fundamental syntax in Rust:
// - variable declaration
// - string manipulation
// - conditional statements
// - loops
// - vectors
// - files
// - user input
// We've tried to limit/hide Rust's quirks since we'll discuss those details
// more in depth in the coming lectures.
extern crate rand;
use rand::Rng;
use std::fs;
use std::io;
use std::io::Write;

const NUM_INCORRECT_GUESSES: u32 = 5;
const WORDS_PATH: &str = "words.txt";

fn pick_a_random_word() -> String {
    let file_string = fs::read_to_string(WORDS_PATH).expect("Unable to read file.");
    let words: Vec<&str> = file_string.split('\n').collect();
    String::from(words[rand::thread_rng().gen_range(0, words.len())].trim())
}

fn main() {
    let secret_word = pick_a_random_word();
    // Note: given what you know about Rust so far, it's easier to pull characters out of a
    // vector than it is to pull them out of a string. You can get the ith character of
    // secret_word by doing secret_word_chars[i].
    let secret_word_chars: Vec<char> = secret_word.chars().collect();
    // Uncomment for debugging:
    println!("random word: {}", secret_word);

    // Your code here! :)
    let mut guessed_or_not = vec![false; secret_word_chars.len()];
    let mut guessed_chars = String::new();
    println!("Welcome to CS110L Hangman!");
    let mut i = 5;
    loop {
        let mut word_so_far = String::new();
        let mut loop_idx = 0;
        while loop_idx < secret_word_chars.len() {
            let cur = String::from(secret_word_chars[loop_idx]);
            word_so_far.push_str(if guessed_or_not[loop_idx] {&cur} else { "-" });
            loop_idx += 1;
        }
        println!("The word so far is {}", word_so_far);
        println!("You have guessed the following letters: {}", guessed_chars);
        println!("You have {} guesses left", i);
        println!("Please guess a letter: ");
        io::stdout()
            .flush()
            .expect("Error flushing stdout.");
        let mut guess = String::new();
        io::stdin()
            .read_line(&mut guess)
            .expect("Error reading line.")
            ;
        guessed_chars.push_str(&guess);
        let mut index = 0;
        let mut found = false;
        while index < secret_word_chars.len() {
            
            let char_vec: Vec<char> = guess.chars().collect();
            if secret_word_chars[index] == char_vec[0] && !guessed_or_not[index] {
                guessed_or_not[index] = true;
                found = true;
                break;
            }

            index += 1;
        }

        if !found {
            i -= 1;
            println!("Sorry, that letter is not in the word");
            if i == 0 {
                // Break
                println!("Sorry, you ran out of guesses!");
                break;
            }
        }
        
        let mut finished = true;
        for bool_iter in guessed_or_not.iter() {
            if !bool_iter {
                finished = false;
            } 
        }

        if finished {
            println!("You have win!");
            break;
        }
        
    }
}
