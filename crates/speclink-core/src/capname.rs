//! capability 命名知識（design D4）：近似名排序與建議池組裝。排序是純函式；
//! 建議池唯讀組合既有 Store 介面（正典列舉、delta 列舉、規格讀取），不寫入。
//! 不含 ANSI；訊息文字由呼叫端（newcmd 的錯誤訊息、validate 的 warning）組裝，
//! 建議行的共用格式在 [`suggestion_line`]。
//!
//! 排序依序比較：名稱 token 的完全包含關係（`auth` ⊂ `authentication`）優先，
//! 其次 kebab 字段交集數，再次編輯距離。入選門檻＝有任何交集（token 包含或
//! 共字段），編輯距離只排序不入選——毫無交集回空清單，拒絕與否由呼叫端的
//! 二元事實（正典有無收錄）決定。比對一律先做 ASCII 小寫折疊：大小寫不敏感
//! 檔案系統上的 `Auth` 與正典 `auth` 必須互相看得見。

use crate::model;
use crate::store::Store;

/// 建議的來源標注。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// 正典規格（openspec/specs/）。
    Canonical,
    /// 未封存 change 的 delta，值＝該 change 名。
    InFlight(String),
}

/// 帶來源標注的既有 capability 名（建議池的一筆，也是建議清單的一筆）。
#[derive(Debug, Clone)]
pub struct KnownName {
    pub name: String,
    pub source: Source,
    /// 該規格 `## Purpose` 區段首行；來源無 Purpose 時為 None（訊息略去該行）。
    pub purpose: Option<String>,
}

/// 候選名對既有名集合的近似建議：至多三筆，無近似回空。
pub fn suggest(candidate: &str, known: &[KnownName]) -> Vec<KnownName> {
    let cand = candidate.to_ascii_lowercase();
    let cand_tokens = tokens(&cand);
    let mut ranked: Vec<(bool, usize, usize, KnownName)> = known
        .iter()
        .filter_map(|k| {
            let name = k.name.to_ascii_lowercase();
            let name_tokens = tokens(&name);
            let contains = token_containment(&cand_tokens, &name_tokens);
            let overlap = name_tokens.iter().filter(|t| cand_tokens.contains(t)).count();
            if !contains && overlap == 0 {
                return None;
            }
            Some((contains, overlap, edit_distance(&cand, &name), k.clone()))
        })
        .collect();
    ranked.sort_by(|a, b| {
        b.0.cmp(&a.0) // 包含優先
            .then(b.1.cmp(&a.1)) // 交集數多者在前
            .then(a.2.cmp(&b.2)) // 編輯距離近者在前
            .then(a.3.name.cmp(&b.3.name)) // 全同時以名稱定序，輸出可重現
    });
    ranked.into_iter().take(3).map(|(_, _, _, k)| k).collect()
}

/// 近似名建議池（design D3）：正典 capabilities（附正典 Purpose 首行）＋所有
/// 未封存 change 的 delta capabilities（附來源 change 名與 delta Purpose 首行），
/// 同名以正典優先去重。建立點主閘與 validate 的 warning lint 共用這一份，
/// 兩處建議一致；validate 端自行濾掉受檢 capability 自身。
pub(crate) fn suggestion_pool(store: &dyn Store) -> Vec<KnownName> {
    let mut caps = store.list_canonical_capabilities();
    caps.sort();
    let mut pool: Vec<KnownName> = caps
        .iter()
        .map(|cap| KnownName {
            name: cap.clone(),
            source: Source::Canonical,
            purpose: purpose_first_line(store.read_canonical_spec(cap).as_deref()),
        })
        .collect();
    for change in store.list_changes() {
        for cap in store.delta_capabilities(&change.name) {
            if pool.iter().any(|k| k.name == cap) {
                continue;
            }
            let text = store.read_artifact(&change.name, &model::delta_spec_artifact(&cap));
            pool.push(KnownName {
                name: cap,
                source: Source::InFlight(change.name.clone()),
                purpose: purpose_first_line(text.as_deref()),
            });
        }
    }
    pool
}

/// 一筆建議的顯示行（不含縮排與列點符號）：`name (來源標注): Purpose 首行`，
/// 無 Purpose 時略去冒號後段。
pub(crate) fn suggestion_line(k: &KnownName) -> String {
    let source = match &k.source {
        Source::Canonical => "canonical".to_string(),
        Source::InFlight(change) => format!("in-flight: {change}"),
    };
    match &k.purpose {
        Some(p) => format!("{} ({source}): {p}", k.name),
        None => format!("{} ({source})", k.name),
    }
}

/// 規格 `## Purpose` 區段的首行（trim 後）；無 Purpose 為 None。
fn purpose_first_line(text: Option<&str>) -> Option<String> {
    model::purpose_content(text?).map(|c| c.lines().next().unwrap_or("").trim().to_string())
}

/// token 層級的完全包含：兩名各自的 kebab 字段中，存在一對相異字段互為子字串
/// （`auth` ⊂ `authentication`）。被包含側須達 3 字元——單字元或兩字元的退化
/// 候選會與幾乎所有既有名成立子字串關係，只產雜訊。
fn token_containment(a: &[&str], b: &[&str]) -> bool {
    a.iter().any(|x| {
        b.iter().any(|y| {
            x != y && x.len().min(y.len()) >= 3 && (x.contains(*y) || y.contains(*x))
        })
    })
}

/// kebab 名稱的字段（去空段——連字號異常時不產生空 token）。
fn tokens(name: &str) -> Vec<&str> {
    name.split('-').filter(|t| !t.is_empty()).collect()
}

/// Levenshtein 編輯距離（字元計），單列 DP。
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut prev = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = if ca == cb { prev } else { prev + 1 };
            prev = row[j + 1];
            row[j + 1] = cost.min(row[j] + 1).min(prev + 1);
        }
    }
    row[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canonical(name: &str) -> KnownName {
        KnownName { name: name.to_string(), source: Source::Canonical, purpose: None }
    }

    fn names(suggestions: &[KnownName]) -> Vec<&str> {
        suggestions.iter().map(|s| s.name.as_str()).collect()
    }

    #[test]
    fn containment_ranks_first() {
        // spec Scenario「包含關係排在最前」：正典有 auth 與 author-tools，
        // 候選 authentication —— auth ⊂ authentication 的包含關係排首位。
        let known = [canonical("author-tools"), canonical("auth")];
        let out = suggest("authentication", &known);
        assert_eq!(names(&out).first(), Some(&"auth"), "包含關係排首位: {out:?}");
    }

    #[test]
    fn token_overlap_beats_edit_distance() {
        // 交集 2 的 alpha-beta-zzzzzzzzzzzzzzzz 編輯距離遠大於交集 1 的
        // alpha-qqqq——交集數優先於編輯距離。
        let known = [canonical("alpha-qqqq"), canonical("alpha-beta-zzzzzzzzzzzzzzzz")];
        let out = suggest("alpha-beta-gamma", &known);
        assert_eq!(
            names(&out),
            vec!["alpha-beta-zzzzzzzzzzzzzzzz", "alpha-qqqq"],
            "交集數勝過編輯距離: {out:?}"
        );
    }

    #[test]
    fn edit_distance_orders_equal_overlap() {
        // 同為交集 1 時，編輯距離近者在前。
        let known = [canonical("gate-zzzzzzzzzzzzzzzz"), canonical("gate-flow")];
        let out = suggest("gate-flows-x", &known);
        assert_eq!(
            names(&out),
            vec!["gate-flow", "gate-zzzzzzzzzzzzzzzz"],
            "編輯距離近者在前: {out:?}"
        );
    }

    #[test]
    fn output_is_capped_at_three() {
        // spec：輸出 SHALL 至多三筆——第四筆近似名被截掉。
        let known = [
            canonical("token-a"),
            canonical("token-b"),
            canonical("token-c"),
            canonical("token-d"),
        ];
        let out = suggest("token-rotation", &known);
        assert_eq!(out.len(), 3, "上限三筆: {out:?}");
    }

    #[test]
    fn no_overlap_yields_an_empty_list() {
        // spec Scenario「無近似名仍拒絕」的排序半部：毫無交集回空清單。
        let known = [canonical("auth"), canonical("review-station")];
        let out = suggest("zzz-unrelated", &known);
        assert!(out.is_empty(), "毫無交集回空: {out:?}");
    }

    #[test]
    fn source_and_purpose_ride_along() {
        // 來源標注與 Purpose 首行原樣進入建議——呼叫端據此組訊息。
        let known = [KnownName {
            name: "user-auth".to_string(),
            source: Source::InFlight("add-sso".to_string()),
            purpose: Some("使用者驗證。".to_string()),
        }];
        let out = suggest("user-authentication", &known);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].source, Source::InFlight("add-sso".to_string()));
        assert_eq!(out[0].purpose.as_deref(), Some("使用者驗證。"));
    }

    #[test]
    fn matching_folds_ascii_case() {
        // 大小寫不敏感檔案系統上 `Auth` 能寫進磁碟，但建議不能因大小寫而
        // 消失：比對折疊、輸出保留原名。
        let known = [canonical("auth")];
        let out = suggest("Auth", &known);
        assert_eq!(names(&out), vec!["auth"], "折疊後互相看得見: {out:?}");
        let reversed = suggest("authentication", &[canonical("AUTH")]);
        assert_eq!(names(&reversed), vec!["AUTH"], "既有名側同樣折疊: {reversed:?}");
    }

    #[test]
    fn degenerate_candidates_produce_no_containment_noise() {
        // 單字元／雙字元候選與幾乎所有既有名都成立子字串關係——3 字元
        // 門檻擋下退化雜訊；純連字號連 token 都切不出來。
        for degenerate in ["a", "ab", "-"] {
            let out = suggest(degenerate, &[canonical("auth"), canonical("archive-merge")]);
            assert!(out.is_empty(), "退化候選 {degenerate:?} 回空: {out:?}");
        }
    }

    #[test]
    fn containment_is_token_level_not_whole_name() {
        // token 層級：candidate 的某字段與既有名的某字段互為子字串才算，
        // 相等字段屬交集不屬包含。user-auth vs user-authentication：
        // auth ⊂ authentication 成立包含，排在僅共 user 一段的 user-flow 前。
        let known = [canonical("user-flow"), canonical("user-authentication")];
        let out = suggest("user-auth", &known);
        assert_eq!(
            names(&out),
            vec!["user-authentication", "user-flow"],
            "token 包含優先於單純交集: {out:?}"
        );
    }

    #[test]
    fn suggestion_line_formats_source_and_purpose() {
        let with_purpose = KnownName {
            name: "auth".into(),
            source: Source::Canonical,
            purpose: Some("Auth session lifecycle.".into()),
        };
        assert_eq!(suggestion_line(&with_purpose), "auth (canonical): Auth session lifecycle.");
        let bare = KnownName {
            name: "user-auth".into(),
            source: Source::InFlight("add-sso".into()),
            purpose: None,
        };
        assert_eq!(suggestion_line(&bare), "user-auth (in-flight: add-sso)");
    }
}
