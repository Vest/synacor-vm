/* Solution for coins:
You stand in the massive central hall of these ruins.  The walls are crumbling, and vegetation has
clearly taken over.  Rooms are attached in all directions.  There is a strange monument in the center
of the hall with circular slots and unusual symbols. It reads:

_ + _ * _^2 + _^3 - _ = 399

Things of interest here:
- red coin      == 2
- corroded coin == 3
- shiny coin    == 5
- concave coin  == 7
- blue coin     == 9
 */

use itertools::Itertools;

const EXPRESSION_RESULT: i32 = 399;

#[inline]
fn calc_expression(a: i32, b: i32, c: i32, d: i32, e: i32) -> i32 {
    a + b * c.pow(2) + d.pow(3) - e
}

fn get_coin(coin: i32) -> &'static str {
    match coin {
        2 => "red coin",
        3 => "corroded coin",
        5 => "shiny coin",
        7 => "concave coin",
        9 => "blue coin",

        _ => "unknown coin"
    }
}

fn main() {
    let input: Vec<i32> = vec![2, 3, 5, 7, 9];

    let result = input.into_iter()
        .permutations(5)
        .find(|perm| {
            calc_expression(perm[0], perm[1], perm[2], perm[3], perm[4]) == EXPRESSION_RESULT
        });

    if let Some(coins) = result {
        println!("The answer is: {:?}. They are", coins);
        coins.iter()
            .for_each(|&coin|
                println!("- {}", get_coin(coin))
            );
    } else {
        println!("Sorry, couldn't find an answer");
    }
}
