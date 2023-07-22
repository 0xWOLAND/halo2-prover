use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
fn hexToDecimal(hex: &str) -> String {
    let mut dec: Vec<i32> = Vec::new();

    for c in hex.chars() {
        let mut carry = i32::from_str_radix(&c.to_string(), 16).unwrap();

        for i in 0..dec.len() {
            let val = dec[i] * 16 + carry;
            dec[i] = val % 10;
            carry = val / 10;
        }

        while (carry > 0) {
            dec.push(carry % 10);
            carry /= 10;
        }
    }

    dec.reverse();

    let res: String = dec
        .iter()
        .fold(String::new(), |mut acc, cur| {
            acc.push_str(&cur.to_string());
            acc
        })
        .to_owned();
    res
}

pub fn unstringifyHex(s: &str) -> String {
    hexToDecimal(
        &general_purpose::STANDARD
            .decode(s)
            .unwrap()
            .iter()
            .map(|x| format!("{:0>2}", &format!("{:x}", x)))
            .fold(String::new(), |mut acc, cur| {
                acc.push_str(&cur);
                acc
            }),
    )
}

#[cfg(test)]
mod test {
    use crate::unstringify::unstringifyHex;

    #[test]
    fn test() {
        assert_eq!(
            "4417881134626180770308697923359573201005643519861877412381846989312604493735",
            unstringifyHex("CcRunsaOm9T+H6q6KUy6OKcaoXdTTN0bbH3A29Cr16c=")
        );
    }
}
