extern crate std;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use num::traits::{Float, Zero};
use num::complex::Complex;

pub fn load_txt<T: FromStr>(filename: &str) -> std::io::Result<Vec<Vec<T>>> {
    // open file
    let file = try!(File::open(filename));

    // read something
    let reader = BufReader::new(file);
    let arr: Vec<Vec<T>> = reader.lines()
        .map(|l| l.unwrap().split(char::is_whitespace)
            .map(|number| number.parse().ok().unwrap())
            .collect())
        .collect();

    Ok(arr)
}

pub fn load_complex<T: FromStr + Float + Zero + Clone>(filename: &str) -> std::io::Result<Vec<Vec<Complex<T>>>> {
    // open file
    let file = try!(File::open(filename));

    // read something
    let reader = BufReader::new(file);
    let arr: Vec<Vec<_>> = reader.lines()
        .map(|l| l.unwrap().split(char::is_whitespace)
            .map(|number| {
                if &number[..1] == "(" && &number[number.len()-1..] == ")" {
                    let number = &number[1..number.len()-1];
                    let numbers: Vec<_> = number.split(',').collect();

                    if numbers.len() == 1 {
                        Complex::<T>::new(numbers[0].parse().ok().unwrap(), T::zero())
                    }
                    else {
                        Complex::<T>::new(numbers[0].parse().ok().unwrap(),
                            numbers[1].parse().ok().unwrap())
                    }
                }
                else {
                    Complex::<T>::new(number.parse().ok().unwrap(), T::zero())
                }
            }).collect())
        .collect();

    Ok(arr)
}

