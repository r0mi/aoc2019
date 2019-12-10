fn valid_password1(pwd:i32) -> bool {
    let mut double:bool = false;
    let mut rem:i32 = pwd % 10;
    let mut _pwd:i32 = pwd / 10;

    loop {
        let new_rem = _pwd % 10;

        if new_rem > rem {
            return false;
        } else if new_rem == rem {
            double = true;
        }

        rem = new_rem;
        _pwd /= 10;

        if _pwd == 0 {
            break;
        }
    }
    double
}

fn valid_password2(pwd:i32) -> bool {
    let mut last_digit:i32 = pwd % 10;
    let mut cur_digit;
    let mut _pwd:i32 = pwd / 10;
    let mut duplicate_count = 1;
    let mut duplicates: Vec<u8> = Vec::new();

    loop {
        cur_digit = _pwd % 10;

        if cur_digit > last_digit {
            return false;
        } else if cur_digit == last_digit {
            duplicate_count += 1
        } else {
            duplicates.push(duplicate_count);
            duplicate_count = 1;
        }

        last_digit = cur_digit;
        _pwd /= 10;

        if _pwd == 0 {
            duplicates.push(duplicate_count);
            break;
        }

    }

    duplicates.iter().any(|l| *l == 2)
}

fn main() {
    let min = 109165;
    let max = 576723;
    let mut valid_passwords1 = 0;
    let mut valid_passwords2 = 0;

    for p in min..max {
        if valid_password1(p) {
            valid_passwords1 += 1;
        }
        if valid_password2(p) {
            valid_passwords2 += 1;
        }
    }

    println!("Total possible passowrds meeting criteria 1 is {:?}", valid_passwords1);
    println!("Total possible passowrds meeting criteria 2 is {:?}", valid_passwords2);
}