//! The door's route table and its path matching (design D1).
//!
//! A pattern has fixed segments, named segments (`{name}`) and, as its last
//! segment only, one that takes the rest of the path (`{*artifact}`). The
//! matching gives the same answer as the demo server's router did:
//!
//! - it works on the path as sent, so an encoded slash (`%2F`) stays inside
//!   its segment; the params are decoded later;
//! - a named segment and the rest of the path are never empty, so a trailing
//!   slash or an empty segment matches nothing;
//! - where two patterns match, the one with a fixed segment first wins
//!   (`/discussions/search` over `/discussions/{slug}`), and a method that
//!   the winning pattern does not have is no match: there is no fall back to
//!   the other pattern;
//! - HEAD takes the GET routes.

use super::routes::{self, Handler};
use super::Method;

/// Every project path starts here; the segment after it is the project key.
pub(crate) const PROJECTS: &str = "/api/speclink/v1/projects/";

/// One route: a method, a pattern under [`PROJECTS`]`{key}`, and its handler.
pub(crate) struct Route {
    pub method: Method,
    pub pattern: &'static str,
    pub handler: Handler,
}

const fn route(method: Method, pattern: &'static str, handler: Handler) -> Route {
    Route { method, pattern, handler }
}

use Method::{Delete, Get, Post, Put};

/// The routes of the door, under `/api/speclink/v1/projects/{key}`. The demo
/// server keeps `/import` and `/events` for itself.
pub(crate) static ROUTES: &[Route] = &[
    route(Get, "/binding", routes::binding),
    route(Get, "/changes", routes::list_changes),
    route(Post, "/changes", routes::create_change),
    route(Get, "/changes/{name}", routes::get_change),
    route(Delete, "/changes/{name}", routes::delete_change),
    route(Get, "/changes/{name}/drift", routes::drift),
    route(Get, "/changes/{name}/validate", routes::validate_change),
    route(Get, "/changes/{name}/analyze", routes::analyze_change),
    route(Post, "/changes/{name}/tasks/move", routes::move_task),
    route(Get, "/changes/{name}/instructions/{*artifact}", routes::instructions),
    route(Get, "/changes/{name}/artifacts/{*artifact}", routes::get_artifact),
    route(Put, "/changes/{name}/artifacts/{*artifact}", routes::put_artifact),
    route(Get, "/changes/{name}/evidence", routes::change_evidence),
    route(Post, "/changes/{name}/tasks/{task_id}/done", routes::task_done),
    route(Post, "/changes/{name}/tasks/{task_id}/undone", routes::task_undone),
    route(Get, "/changes/{name}/review", routes::review_show),
    route(Delete, "/changes/{name}/review", routes::review_discard),
    route(Post, "/changes/{name}/review/rounds", routes::review_add_round),
    route(Post, "/changes/{name}/review/stamp", routes::review_stamp),
    route(Get, "/changes/{name}/verify", routes::verify_show),
    route(Delete, "/changes/{name}/verify", routes::verify_discard),
    route(Post, "/changes/{name}/verify/rounds", routes::verify_add_round),
    route(Post, "/changes/{name}/verify/stamp", routes::verify_stamp),
    route(Post, "/changes/{name}/claim", routes::claim),
    route(Post, "/changes/{name}/depends", routes::change_depends),
    route(Post, "/changes/{name}/schema", routes::change_schema),
    route(Post, "/changes/{name}/rank", routes::change_rank),
    route(Post, "/changes/{name}/in-progress", routes::in_progress),
    route(Delete, "/changes/{name}/in-progress", routes::in_progress_remove),
    route(Post, "/changes/{name}/archive", routes::archive),
    route(Get, "/discussions", routes::list_discussions),
    route(Post, "/discussions", routes::create_discussion),
    route(Get, "/discussions/search", routes::search_discussions),
    route(Get, "/discussions/{slug}", routes::show_discussion),
    route(Delete, "/discussions/{slug}", routes::delete_discussion),
    route(Post, "/discussions/{slug}/link", routes::link_discussion),
    route(Post, "/discussions/{slug}/seal", routes::seal_discussion),
    route(Put, "/discussions/{slug}/context", routes::set_discussion_context),
    route(Post, "/discussions/{slug}/rounds", routes::add_discussion_round),
    route(Post, "/discussions/{slug}/conclude", routes::conclude_discussion),
    route(Post, "/discussions/{slug}/archive", routes::archive_discussion),
    route(Post, "/discussions/{slug}/promote", routes::promote_discussion),
    route(Get, "/plan", routes::plan),
    route(Get, "/specs", routes::list_specs),
    route(Get, "/specs/{capability}/document", routes::spec_document),
    route(Get, "/archived", routes::archived_list),
    route(Get, "/archived/{dated_name}/artifacts/{*artifact}", routes::archived_artifact),
    route(Get, "/archived/{dated_name}/capabilities", routes::archived_capabilities),
    route(Get, "/search", routes::search),
    route(Get, "/language", routes::language),
    route(Get, "/config", routes::config),
    route(Put, "/config", routes::put_config),
    route(Get, "/board-order", routes::board_order),
    route(Put, "/board-order", routes::put_board_order),
    route(Get, "/whoami", routes::whoami),
    route(Get, "/sync-state", routes::sync_state),
    route(Post, "/context", routes::context),
];

/// The POST routes that only read: a caller who cannot write still uses
/// them. A new one goes here, and in the design.
pub(crate) const READ_ONLY_POSTS: &[&str] = &["/context"];

/// What a path matched: the route's pattern and its params, as they are in
/// the path (still percent-encoded), the project key first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteMatch {
    pub pattern: &'static str,
    pub params: Vec<String>,
}

/// The door's route that `method` and `path` (the whole request path)
/// reach, if any. A host can use it to tell the door's paths apart before
/// it reads a body, for example to choose a size limit.
pub fn match_route(method: Method, path: &str) -> Option<RouteMatch> {
    find(method, path).map(|(route, params)| RouteMatch { pattern: route.pattern, params })
}

/// The method and the pattern of every route, patterns under
/// `/api/speclink/v1/projects/{key}`.
pub fn routes() -> impl Iterator<Item = (Method, &'static str)> {
    ROUTES.iter().map(|route| (route.method, route.pattern))
}

/// How a pattern segment matched; a later variant wins over an earlier one
/// at the first position where two matches differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Rest,
    Named,
    Fixed,
}

/// The route and the raw params (project key first) of `method` and `path`.
pub(crate) fn find(method: Method, path: &str) -> Option<(&'static Route, Vec<String>)> {
    let segments: Vec<&str> = path.strip_prefix(PROJECTS)?.split('/').collect();
    let (key, rest) = segments.split_first()?;
    if key.is_empty() {
        return None;
    }
    // The best pattern for the path, whatever the method.
    let mut best: Option<(Vec<Kind>, &'static str, Vec<String>)> = None;
    for route in ROUTES {
        if best.as_ref().is_some_and(|(_, pattern, _)| *pattern == route.pattern) {
            continue;
        }
        let Some((kinds, params)) = match_pattern(route.pattern, rest) else {
            continue;
        };
        if best.as_ref().is_none_or(|(best_kinds, _, _)| kinds > *best_kinds) {
            best = Some((kinds, route.pattern, params));
        }
    }
    let (_, pattern, params) = best?;
    let wanted = if method == Method::Head { Method::Get } else { method };
    let route = ROUTES.iter().find(|route| route.pattern == pattern && route.method == wanted)?;
    let mut all = Vec::with_capacity(params.len() + 1);
    all.push(key.to_string());
    all.extend(params);
    Some((route, all))
}

/// How `pattern` matches the path segments, and the params it takes.
fn match_pattern(pattern: &str, segments: &[&str]) -> Option<(Vec<Kind>, Vec<String>)> {
    let parts: Vec<&str> = pattern.trim_start_matches('/').split('/').collect();
    let mut kinds = Vec::with_capacity(parts.len());
    let mut params = Vec::new();
    for (i, part) in parts.iter().enumerate() {
        if part.starts_with("{*") {
            let rest = segments.get(i..).unwrap_or_default().join("/");
            if rest.is_empty() {
                return None;
            }
            kinds.push(Kind::Rest);
            params.push(rest);
            return Some((kinds, params));
        }
        let segment = segments.get(i)?;
        if part.starts_with('{') {
            if segment.is_empty() {
                return None;
            }
            kinds.push(Kind::Named);
            params.push(segment.to_string());
        } else if part == segment {
            kinds.push(Kind::Fixed);
        } else {
            return None;
        }
    }
    (segments.len() == parts.len()).then_some((kinds, params))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pattern_has_one_route_per_method() {
        let mut seen = std::collections::BTreeSet::new();
        for route in ROUTES {
            assert!(seen.insert((route.pattern, route.method.as_str())), "{} {}", route.method.as_str(), route.pattern);
        }
    }

    #[test]
    fn the_rest_segment_is_only_ever_last() {
        for route in ROUTES {
            let parts: Vec<&str> = route.pattern.split('/').collect();
            for part in &parts[..parts.len() - 1] {
                assert!(!part.starts_with("{*"), "{}", route.pattern);
            }
        }
    }
}
