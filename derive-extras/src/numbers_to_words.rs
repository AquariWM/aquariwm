// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::iter;

const DIGITS: [&str; 20] = [
	"zero",
	"one",
	"two",
	"three",
	"four",
	"five",
	"six",
	"seven",
	"eight",
	"nine",
	"ten",
	"eleven",
	"twelve",
	"thirteen",
	"fourteen",
	"fifteen",
	"sixteen",
	"seventeen",
	"eighteen",
	"nineteen",
];
const TENS: [&str; 8] = [
	"twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
];
const ORDERS: [&str; 12] = [
	"thousand",
	"million",
	"billion",
	"trillion",
	"quadrillion",
	"quintillion",
	"sextillion",
	"septillion",
	"octillion",
	"nonillion",
	"decillion",
	"undecillion",
];

const DIGITS_ORDINAL: [&str; 20] = [
	"zeroth",
	"first",
	"second",
	"third",
	"fourth",
	"fifth",
	"sixth",
	"seventh",
	"eighth",
	"ninth",
	"tenth",
	"eleventh",
	"twelfth",
	"thirteenth",
	"fourteenth",
	"fifteenth",
	"sixteenth",
	"seventeenth",
	"eighteenth",
	"nineteenth",
];
const TENS_ORDINAL: [&str; 8] = [
	"twentieth",
	"thirtieth",
	"fortieth",
	"fiftieth",
	"sixtieth",
	"seventieth",
	"eightieth",
	"ninetieth",
];
const ORDERS_ORDINAL: [&str; 12] = [
	"thousandth",
	"millionth",
	"billionth",
	"trillionth",
	"quadrillionth",
	"quintillionth",
	"sextillionth",
	"septillionth",
	"octillionth",
	"nonillionth",
	"decillionth",
	"undecillionth",
];

/// Encodes the given `number` as a [string] using the given `separator` between words.
///
/// # Examples
/// ```no_run
/// # fn encode(number: usize, separator: char) -> String { "".to_owned() }
/// assert_eq!(encode(1_782, ' '), "one thousand seven hundred and eighty two");
/// assert_eq!(encode(93, '_'), "ninety_three");
/// ```
///
/// [string]: String
pub fn encode(number: usize, separator: char) -> String {
	let sep = separator;

	match number {
		// Digits and teens.
		0..=19 => DIGITS[number].to_owned(),
		// Other tens.
		20..=99 => {
			// Tens start at twenty, so subtract 2.
			let tens = (number / 10) - 2;
			let digits = number % 10;

			match digits {
				0 => TENS[tens].to_owned(),
				_ => format!("{}{sep}{}", TENS[tens], DIGITS[digits]),
			}
		},
		// Hundreds.
		100..=999 => {
			let hundreds = number / 100;
			let rest = number % 100;

			match rest {
				0 => format!("{}{sep}hundred", DIGITS[hundreds]),
				rest => format!("{}{sep}hundred{sep}and{}", DIGITS[hundreds], encode(rest, sep)),
			}
		},
		// Other numbers.
		_ => {
			let (div, order) = iter::successors(Some(1usize), |n| n.checked_mul(1_000))
				.zip(ORDERS)
				.find(|&(n, _)| n > number / 1_000)
				.unwrap();

			let upper = number / div;
			let rest = number % div;

			match rest {
				0 => format!("{}{sep}{}", encode(upper, sep), order),
				_ => format!("{}{sep}{}{sep}{}", encode(upper, sep), order, encode(rest, sep)),
			}
		},
	}
}

/// Encodes the given ordinal `number` as a [string] using the given `separator` between words.
///
/// # Examples
/// ```no_run
/// # fn encode_ordinal(number: usize, separator: char) -> String { "".to_owned() }
/// assert_eq!(encode_ordinal(1_782, ' '), "one thousand seven hundred and eighty second");
/// assert_eq!(encode_ordinal(93, '_'), "ninety_third");
/// ```
///
/// [string]: String
pub fn encode_ordinal(number: usize, separator: char) -> String {
	let sep = separator;

	match number {
		// Digits and teens.
		0..=19 => DIGITS_ORDINAL[number].to_owned(),
		// Other tens.
		20..=99 => {
			// Tens start at twenty, so subtract 2.
			let tens = (number / 10) - 2;
			let digits = number % 10;

			match digits {
				0 => TENS_ORDINAL[tens].to_owned(),
				_ => format!("{}{sep}{}", TENS[tens], DIGITS_ORDINAL[digits]),
			}
		},
		// Hundreds.
		100..=999 => {
			let hundreds = number / 100;
			let rest = number % 100;

			match rest {
				0 => format!("{}{sep}hundredth", DIGITS[hundreds]),
				_ => format!(
					"{}{sep}hundred{sep}and{sep}{}",
					DIGITS[hundreds],
					encode_ordinal(rest, sep)
				),
			}
		},
		// Other numbers.
		_ => {
			let (div, (order, order_ordinal)) = iter::successors(Some(1usize), |n| n.checked_mul(1_000))
				.zip(ORDERS.iter().zip(ORDERS_ORDINAL))
				.find(|&(n, _)| n > number / 1_000)
				.unwrap();

			let upper = number / div;
			let rest = number % div;

			match rest {
				0 => format!("{}{sep}{}", encode(upper, sep), order_ordinal),
				_ => format!("{}{sep}{}{sep}{}", encode(upper, sep), order, encode_ordinal(rest, sep)),
			}
		},
	}
}
