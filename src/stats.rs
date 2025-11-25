pub fn variance(input: &[f64]) -> Option<f64> {
    if input.is_empty() {
        return None;
    }
    let len = input.len() as f64;
    let mean = kahan_babushka_neumaier_sum(input.iter().copied()) / len;
    Some(
        kahan_babushka_neumaier_sum(input.iter().map(|x| {
            let d = x - mean;
            d * d
        })) / len,
    )
}

fn kahan_babushka_neumaier_sum(input: impl IntoIterator<Item = f64>) -> f64 {
    let (sum, c) = input.into_iter().fold((0f64, 0f64), |(sum, c), x| {
        let t = sum + x;
        let d = if sum.abs() >= x.abs() {
            (sum - t) + x
        } else {
            (x - t) + sum
        };
        (t, c + d)
    });
    sum + c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variance() {
        assert_eq!(variance(&[1.0, 2.0, 3.0]), Some(2.0 / 3.0));
    }

    #[test]
    fn test_kahan_babushka() {
        assert_eq!(kahan_babushka_neumaier_sum(vec![1.0, 2.0, 3.0]), 6.0);
    }

    #[test]
    fn test_kahan_babushka_large() {
        assert_eq!([1e100, 1.0, -1e100].iter().sum::<f64>(), 0.0);
        assert_eq!(kahan_babushka_neumaier_sum(vec![1e100, 1.0, -1e100]), 1.0);
    }
}
