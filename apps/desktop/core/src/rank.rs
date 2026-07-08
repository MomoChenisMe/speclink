//! 看板卡片排序鍵演算（design D1）：字串型 fractional key，不用浮點。
//!
//! 鍵為小寫英文字母字串，視為 base-26 分數（'a'=0 … 'z'=25），字典序＝數值序
//! （生成鍵永不以 'a' 結尾，避免同值異字串）。中點取前後鄰居的字典序中位，
//! 無縫隙時延長鍵長——以延長取代重平衡，重平衡機制整個不需要。

const BASE: u32 = 26;

/// 第 `i` 位數值：下界鍵超出長度視為 0（分數尾端補零）。
fn digit(s: &str, i: usize) -> u32 {
    s.as_bytes().get(i).map_or(0, |b| u32::from(b - b'a'))
}

/// 上界鍵第 `i` 位數值：`None`（欄底＝無上界）視為每一位皆 BASE。
fn upper_digit(s: Option<&str>, i: usize) -> u32 {
    match s {
        None => BASE,
        Some(s) => digit(s, i),
    }
}

/// 兩鄰居之間的中點鍵：嚴格介於 `prev` 與 `next` 之間（字典序）。
/// `prev`＝None 表欄頂（下界為零）、`next`＝None 表欄底（無上界）；兩者皆 None
/// 產生首鍵。呼叫端保證 prev < next；鍵縫隙不足時延長鍵長，不改寫任何鄰居。
pub fn midpoint(prev: Option<&str>, next: Option<&str>) -> String {
    let a = prev.unwrap_or("");
    let mut out = String::new();
    let mut i = 0;
    // 沿共同前綴走到第一個分歧位；分歧位有縫隙即取中位收束。
    loop {
        let da = digit(a, i);
        let db = upper_digit(next, i);
        if da == db {
            out.push((b'a' + da as u8) as char);
            i += 1;
            continue;
        }
        let mid = (da + db) / 2;
        if mid > da {
            out.push((b'a' + mid as u8) as char);
            return out;
        }
        // db == da + 1：本位取 da（已嚴格小於上界），其後不再受上界約束，
        // 只需產生嚴格大於下界餘位的尾段。
        out.push((b'a' + da as u8) as char);
        i += 1;
        loop {
            let da = digit(a, i);
            let mid = (da + BASE) / 2;
            if mid > da {
                out.push((b'a' + mid as u8) as char);
                return out;
            }
            // da == 'z'：無法在本位放大，照抄後續位。
            out.push((b'a' + da as u8) as char);
            i += 1;
        }
    }
}

/// 批次派發 `n` 個嚴格遞增、兩兩留有可再分縫隙的鍵（整欄補章用，design D3）。
/// 鍵在 base-26^width 值域內等距取點（width 取到值域 ≥ 2(n+1)，保證步距 ≥ 2）；
/// 值恰為 26 的倍數時尾位會成零位 'a'，加一避開（步距 ≥ 2 保證仍嚴格遞增）。
pub fn spread(n: usize) -> Vec<String> {
    if n == 0 {
        return Vec::new();
    }
    let mut width = 1u32;
    let mut total: u64 = u64::from(BASE);
    while total < 2 * (n as u64 + 1) {
        width += 1;
        total *= u64::from(BASE);
    }
    let step = total / (n as u64 + 1);
    (1..=n as u64)
        .map(|i| {
            let mut v = i * step;
            if v % u64::from(BASE) == 0 {
                v += 1;
            }
            // 以固定 width 位數渲染（高位在前）。
            let mut digits = vec![b'a'; width as usize];
            let mut rest = v;
            for d in digits.iter_mut().rev() {
                *d = b'a' + (rest % u64::from(BASE)) as u8;
                rest /= u64::from(BASE);
            }
            String::from_utf8(digits).expect("base-26 digits are ASCII")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{midpoint, spread};

    /// 生成鍵的合法形狀：非空、僅小寫字母、不以 'a'（零位）結尾。
    fn assert_valid_key(k: &str) {
        assert!(!k.is_empty(), "key must be non-empty");
        assert!(
            k.bytes().all(|b| b.is_ascii_lowercase()),
            "key must be lowercase ASCII letters only: {k:?}"
        );
        assert!(!k.ends_with('a'), "key must not end with the zero digit 'a': {k:?}");
    }

    #[test]
    fn midpoint_is_strictly_between_gapped_neighbors() {
        // spec Example「中點與延長」第 1 列：有縫隙取中點。
        let m = midpoint(Some("b"), Some("f"));
        assert_valid_key(&m);
        assert!("b" < m.as_str() && m.as_str() < "f", "b < {m} < f violated");
    }

    #[test]
    fn midpoint_extends_length_when_neighbors_are_adjacent() {
        // spec Example「中點與延長」第 2 列：ab 與 ac 無縫隙——以 ab 為前綴延長。
        let m = midpoint(Some("ab"), Some("ac"));
        assert_valid_key(&m);
        assert!("ab" < m.as_str() && m.as_str() < "ac", "ab < {m} < ac violated");
        assert!(m.starts_with("ab"), "adjacent keys must extend the lower key: {m}");

        let m2 = midpoint(Some("b"), Some("c"));
        assert_valid_key(&m2);
        assert!("b" < m2.as_str() && m2.as_str() < "c");
    }

    #[test]
    fn midpoint_handles_open_ends() {
        // spec Example 第 3、4 列：欄頂／欄底單側推導。
        let top = midpoint(None, Some("b"));
        assert_valid_key(&top);
        assert!(top.as_str() < "b", "top insert must be strictly less than b: {top}");

        let bottom = midpoint(Some("n"), None);
        assert_valid_key(&bottom);
        assert!(bottom.as_str() > "n", "bottom insert must be strictly greater than n: {bottom}");

        // 首鍵（空欄第一次寫入）。
        assert_valid_key(&midpoint(None, None));
    }

    #[test]
    fn midpoint_property_holds_across_generated_pairs() {
        // 性質測試：對 spread 生成集的任兩相鄰與跨距鍵，中點嚴格介於其間。
        let keys = spread(12);
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                let (a, b) = (&keys[i], &keys[j]);
                let m = midpoint(Some(a), Some(b));
                assert_valid_key(&m);
                assert!(
                    a.as_str() < m.as_str() && m.as_str() < b.as_str(),
                    "{a} < {m} < {b} violated"
                );
            }
        }
    }

    #[test]
    fn repeated_top_insert_keeps_producing_smaller_valid_keys() {
        // 病態連續插入同一縫隙（欄頂）：每次仍嚴格更小、鍵形合法——延長取代重平衡。
        let mut first = midpoint(None, None);
        for _ in 0..60 {
            let smaller = midpoint(None, Some(&first));
            assert_valid_key(&smaller);
            assert!(smaller < first, "{smaller} must sort before {first}");
            first = smaller;
        }
    }

    #[test]
    fn spread_returns_strictly_increasing_subdividable_keys() {
        // 批次派發（整欄補章）：嚴格遞增、鍵形合法、兩兩可再取中點插入。
        for n in [1usize, 2, 3, 10, 30, 200] {
            let keys = spread(n);
            assert_eq!(keys.len(), n, "spread({n}) must return {n} keys");
            for k in &keys {
                assert_valid_key(k);
            }
            for w in keys.windows(2) {
                assert!(w[0] < w[1], "spread keys must be strictly increasing: {w:?}");
                let m = midpoint(Some(&w[0]), Some(&w[1]));
                assert!(
                    w[0] < m && m < w[1],
                    "every adjacent pair must stay subdividable: {} < {m} < {}",
                    w[0],
                    w[1]
                );
            }
        }
        assert!(spread(0).is_empty());
    }
}
